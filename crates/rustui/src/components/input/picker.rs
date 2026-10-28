//! Controlled single-selection picker backed by a retained GPUI SelectState.

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use gpui::{
    AnyElement, App, AppContext as _, Entity, IntoElement, ParentElement as _, SharedString,
    Styled, div, px,
};
use gpui_component::{
    Icon, IconName, IndexPath, Sizable, Size,
    select::{SearchableVec, Select as GpuiSelect, SelectEvent, SelectState},
};
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, selector, styled_element},
    view::{EmptyView, RenderContext, View},
};

type PickerSelectState = SelectState<SearchableVec<super::SelectOption>>;
type PickerSelectionWriter = Rc<dyn Fn(String, &mut App)>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum PickerLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_message_label(
    id: &SharedString,
    localized: String,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.as_ref().trim() {
        fallback.clone()
    } else {
        SharedString::from(localized)
    }
}

struct RetainedPicker {
    select: Entity<PickerSelectState>,
    options: Vec<super::SelectOption>,
    owner_value: String,
    searchable: bool,
    selection_writer: Rc<RefCell<PickerSelectionWriter>>,
}

fn with_current_writer<T: Clone, E, R>(
    writer: &RefCell<T>,
    event: E,
    apply: impl FnOnce(T, E) -> R,
) -> R {
    let writer = writer.borrow().clone();
    apply(writer, event)
}

/// A controlled picker whose selection and dropdown state survive body rebuilds.
pub struct PickerView {
    label: SharedString,
    selection: Binding<String>,
    options: Vec<String>,
    option_keys: std::collections::HashMap<String, SharedString>,
    id: Option<Identity>,
    disabled: bool,
    searchable: bool,
    search_placeholder: Option<SharedString>,
    accessibility_label: Option<PickerLabel>,
    appearance: bool,
}

impl PickerView {
    pub fn new<I, S>(label: impl Into<SharedString>, selection: Binding<String>, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let label = label.into();
        assert!(!label.trim().is_empty(), "Picker label must not be blank");
        Self {
            label,
            selection,
            options: collect_unique_options(options),
            option_keys: std::collections::HashMap::new(),
            id: None,
            disabled: false,
            searchable: false,
            search_placeholder: None,
            accessibility_label: None,
            appearance: true,
        }
    }

    /// Localize an option's displayed title without changing its binding value.
    pub fn localized_option(
        mut self,
        value: impl Into<String>,
        key: impl Into<SharedString>,
    ) -> Self {
        let value = value.into();
        let key = key.into();
        assert!(
            self.options.contains(&value),
            "Picker localized option must exist"
        );
        assert!(
            !key.trim().is_empty(),
            "Picker option localization key must not be blank"
        );
        self.option_keys.insert(value, key);
        self
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Picker IDs must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = Some(placeholder.into());
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Picker accessibility label must not be blank"
        );
        self.accessibility_label = Some(PickerLabel::Literal(label));
        self
    }

    /// Resolves the picker's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let message_id = message_id.into();
        assert!(
            !message_id.trim().is_empty(),
            "Picker accessibility localization keys must be nonempty"
        );
        let fallback = match &self.accessibility_label {
            Some(PickerLabel::Literal(label))
            | Some(PickerLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.label.clone(),
        };
        self.accessibility_label = Some(PickerLabel::Message {
            id: message_id,
            fallback,
        });
        self
    }

    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| allocate_implicit(Identity::from("picker")))
    }
}

#[allow(non_snake_case)]
pub fn Picker<I, S>(
    label: impl Into<SharedString>,
    selection: Binding<String>,
    options: I,
) -> PickerView
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    PickerView::new(label, selection, options)
}

impl View for PickerView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity(context);
        let options = super::select_options(&self.options, &self.option_keys, |key| {
            context.localized_message(key)
        });
        let initial_value = self.selection.value();
        let initial_index = option_index(&self.options, &initial_value);
        let initial_options = options.clone();
        let searchable = self.searchable;
        let selection_writer = context.binding_writer(self.selection.clone());
        let initial_selection_writer = selection_writer.clone();
        let (retained, created) =
            context.retained_entity(identity.clone(), move |window, cx| RetainedPicker {
                select: cx.new(|cx| {
                    SelectState::new(
                        SearchableVec::new(initial_options.clone()),
                        initial_index,
                        window,
                        cx,
                    )
                    .searchable(searchable)
                }),
                options: initial_options,
                owner_value: initial_value,
                searchable,
                selection_writer: Rc::new(RefCell::new(initial_selection_writer)),
            });

        let (
            mut select,
            retained_options,
            previous_owner_value,
            retained_searchable,
            current_selection_writer,
        ) = {
            let retained = retained.read(context.app());
            (
                retained.select.clone(),
                retained.options.clone(),
                retained.owner_value.clone(),
                retained.searchable,
                retained.selection_writer.clone(),
            )
        };
        *current_selection_writer.borrow_mut() = selection_writer;

        let current = self.selection.value();
        let searchable_changed = picker_searchable_changed(retained_searchable, self.searchable);
        if searchable_changed {
            let options = options.clone();
            let selection = current.clone();
            let initial_index = option_index(&self.options, &selection);
            let searchable = self.searchable;
            select = context.update_entity(&retained, move |retained, window, cx| {
                let next = cx.new(|cx| {
                    SelectState::new(SearchableVec::new(options), initial_index, window, cx)
                        .searchable(searchable)
                });
                retained.select = next.clone();
                retained.searchable = searchable;
                next
            });
        }

        let options_changed = retained_options != options;
        if options_changed {
            let options = options.clone();
            context.update_entity(&select, |state, window, cx| {
                state.set_items(SearchableVec::new(options.clone()), window, cx);
            });
            context.update_entity(&retained, move |retained, _window, _cx| {
                retained.options = options;
            });
        }

        let owner_changed = previous_owner_value != current;
        if super::select_needs_reconciliation(options_changed, &previous_owner_value, &current) {
            let value_to_sync = current.clone();
            context.update_entity(&select, move |state, window, cx| {
                state.set_selected_value(&value_to_sync, window, cx);
            });
        }
        if owner_changed {
            context.update_entity(&retained, move |retained, _window, _cx| {
                retained.owner_value = current;
            });
        }

        if created || searchable_changed {
            let current_selection_writer = current_selection_writer.clone();
            let retained_picker = retained.clone();
            context.replace_retained_subscription(
                &identity,
                Identity::from("picker-select-state-events"),
                &select,
                move |state, event: &SelectEvent<SearchableVec<super::SelectOption>>, cx| {
                    if state.entity_id() != retained_picker.read(cx).select.entity_id() {
                        return;
                    }
                    let SelectEvent::Confirm(value) = event;
                    if let Some(value) = value {
                        let confirmed_value = value.clone();
                        retained_picker.update(cx, |retained, _cx| {
                            record_confirmed_selection(&mut retained.owner_value, &confirmed_value);
                        });
                        with_current_writer(
                            &current_selection_writer,
                            value.clone(),
                            |write, value| write(value, cx),
                        );
                    }
                },
            );
        }

        let mut picker = GpuiSelect::new(&select)
            .id(identity.to_string())
            .placeholder(self.label.clone())
            .accessibility_label(match &self.accessibility_label {
                Some(PickerLabel::Literal(label)) => label.clone(),
                Some(PickerLabel::Message { id, fallback }) => {
                    resolved_message_label(id, context.localized_message(id.as_ref()), fallback)
                }
                None => self.label.clone(),
            })
            .disabled(self.disabled)
            .appearance(self.appearance);
        if let Some(placeholder) = &self.search_placeholder {
            picker = picker.search_placeholder(placeholder.clone());
        }
        if !self.appearance {
            return picker.into_any_element();
        }

        // GPUI Select owns focus, keyboard handling, and the menu. Its icon slot
        // has no background styling hook, so draw the themed affordance over an
        // empty icon while leaving the native trigger fully interactive.
        let style = required_style(context, "picker", "body", &selector(context));
        let mut affordance = styled_element(div().flex().items_center().justify_center(), &style)
            .absolute()
            .right(px(9.))
            .top_0()
            .bottom_0()
            .my_auto();
        if self.disabled {
            affordance = affordance.opacity(0.5);
        }
        let affordance = affordance.child(Icon::new(IconName::ChevronDown).with_size(Size::XSmall));
        div()
            .relative()
            .child(picker.icon(Icon::empty()))
            .child(affordance)
            .into_any_element()
    }
}

fn collect_unique_options<I, S>(options: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let options: Vec<String> = options.into_iter().map(Into::into).collect();
    let mut seen = HashSet::new();
    assert!(
        options.iter().all(|option| !option.trim().is_empty()),
        "Picker options must not be blank"
    );
    assert!(
        options.iter().all(|option| seen.insert(option.as_str())),
        "Picker options must be unique"
    );
    options
}

fn option_index(options: &[String], selection: &str) -> Option<IndexPath> {
    options
        .iter()
        .position(|option| option == selection)
        .map(IndexPath::new)
}

fn picker_searchable_changed(previous: bool, current: bool) -> bool {
    previous != current
}

fn record_confirmed_selection(owner_value: &mut String, confirmed_value: &str) {
    *owner_value = confirmed_value.to_owned();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> Binding<String> {
        Binding::new(
            String::new(),
            |value: &String| value.clone(),
            |value, next| *value = next,
        )
    }

    #[test]
    #[should_panic(expected = "Picker label must not be blank")]
    fn rejects_blank_default_accessible_name() {
        Picker(" \t ", binding(), ["One"]);
    }

    #[test]
    #[should_panic(expected = "Picker accessibility label must not be blank")]
    fn rejects_blank_explicit_accessible_name() {
        Picker("Choice", binding(), ["One"]).accessibility_label(" \t ");
    }

    #[test]
    fn preserves_nonblank_default_and_explicit_accessible_names() {
        let default = Picker("  Choice  ", binding(), ["One"]);
        assert_eq!(default.label.as_ref(), "  Choice  ");
        assert_eq!(default.accessibility_label, None);

        let explicit =
            Picker("Choice", binding(), ["One"]).accessibility_label("  Select an option  ");
        assert_eq!(
            explicit.accessibility_label,
            Some(PickerLabel::Literal("  Select an option  ".into()))
        );
    }

    #[test]
    #[should_panic(expected = "Picker options must not be blank")]
    fn rejects_blank_accessible_option_names() {
        Picker("Choice", binding(), ["One", " \t "]);
    }

    #[test]
    fn localized_accessible_name_resolves_message_and_preserves_fallback() {
        let picker = Picker("Choice", binding(), ["One"])
            .accessibility_label("Select an option")
            .localized_accessibility_label("picker.choice");
        let Some(PickerLabel::Message { id, fallback }) = picker.accessibility_label else {
            panic!("expected localized Picker accessibility label");
        };

        assert_eq!(id.as_ref(), "picker.choice");
        assert_eq!(fallback.as_ref(), "Select an option");
        assert_eq!(
            resolved_message_label(&id, "picker.choice".into(), &fallback),
            fallback
        );
        assert_eq!(
            resolved_message_label(&id, "Select a choice".into(), &fallback).as_ref(),
            "Select a choice"
        );
        assert_eq!(
            resolved_message_label(&id, " \t".into(), &fallback),
            fallback
        );
    }

    #[test]
    fn localized_accessible_name_defaults_to_visible_label_fallback() {
        let picker =
            Picker("Choice", binding(), ["One"]).localized_accessibility_label("picker.choice");

        assert!(matches!(
            picker.accessibility_label,
            Some(PickerLabel::Message { ref fallback, .. }) if fallback.as_ref() == "Choice"
        ));
    }

    #[test]
    fn retained_confirmation_uses_the_latest_selection_writer() {
        let writes = Rc::new(RefCell::new(Vec::new()));
        type TestWriter = Rc<dyn Fn(String)>;
        let old_writes = writes.clone();
        let current: RefCell<TestWriter> = RefCell::new(Rc::new(move |value| {
            old_writes.borrow_mut().push(("old", value));
        }));
        let new_writes = writes.clone();
        let new_writer: TestWriter = Rc::new(move |value| {
            new_writes.borrow_mut().push(("new", value));
        });
        *current.borrow_mut() = new_writer.clone();

        with_current_writer(&current, "Swift".to_owned(), |write, value| write(value));

        assert_eq!(&*writes.borrow(), &[("new", "Swift".to_owned())]);
    }

    #[test]
    fn retained_confirmation_releases_writer_slot_before_applying() {
        type TestWriter = Rc<dyn Fn(String)>;
        let current = Rc::new(RefCell::new(Rc::new(|_| {}) as TestWriter));
        let writer_slot = current.clone();
        *current.borrow_mut() = Rc::new(move |_| {
            *writer_slot.borrow_mut() = Rc::new(|_| {});
        });

        with_current_writer(current.as_ref(), "Swift".to_owned(), |write, value| {
            write(value);
        });
    }

    #[test]
    fn selection_reconciliation_tracks_owner_and_option_changes() {
        assert!(!super::super::select_needs_reconciliation(
            false,
            "unmatched",
            "unmatched"
        ));
        assert!(super::super::select_needs_reconciliation(
            false, "old", "new"
        ));
        assert!(super::super::select_needs_reconciliation(
            true, "same", "same"
        ));
    }

    #[test]
    fn rejected_confirmation_reconciles_to_controlled_owner_value() {
        let mut retained_owner_value = String::from("Rust");

        record_confirmed_selection(&mut retained_owner_value, "Swift");

        assert!(super::super::select_needs_reconciliation(
            false,
            &retained_owner_value,
            "Rust"
        ));
    }

    #[test]
    fn searchable_changes_rebuild_retained_select_state() {
        assert!(!picker_searchable_changed(false, false));
        assert!(picker_searchable_changed(false, true));
        assert!(picker_searchable_changed(true, false));
    }

    #[test]
    #[should_panic(expected = "Picker accessibility localization keys must be nonempty")]
    fn rejects_blank_accessible_name_message_id() {
        Picker("Choice", binding(), ["One"]).localized_accessibility_label(" \t");
    }
}

#[cfg(test)]
mod implicit_identity_tests {
    use super::*;

    fn binding() -> Binding<String> {
        Binding::new(
            String::new(),
            |value: &String| value.clone(),
            |value, next| *value = next,
        )
    }

    #[test]
    fn same_label_siblings_get_distinct_stable_implicit_identities() {
        let first = Picker("Choice", binding(), ["One"]);
        let second = Picker("Choice", binding(), ["One"]);
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first_sibling = first.identity_with(allocate_at_slot(0));
        let second_sibling = second.identity_with(allocate_at_slot(1));
        let first_after_rerender = first.identity_with(allocate_at_slot(0));
        let first_after_label_change =
            Picker("Choix", binding(), ["One"]).identity_with(allocate_at_slot(0));

        assert_ne!(first_sibling, second_sibling);
        assert_eq!(first_sibling, first_after_rerender);
        assert_eq!(first_sibling, first_after_label_change);
    }
}
