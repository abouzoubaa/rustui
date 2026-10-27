//! Controlled searchable ComboBox backed by a retained GPUI SelectState.

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

type RetainedComboBoxState = SelectState<SearchableVec<String>>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ComboBoxAccessibilityLabel {
    Explicit(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_accessibility_label(
    id: &str,
    localized: String,
    fallback: &SharedString,
) -> SharedString {
    if !localized.trim().is_empty() && localized.trim() != id.trim() {
        SharedString::from(localized)
    } else {
        fallback.clone()
    }
}

struct RetainedComboBox {
    select: Entity<RetainedComboBoxState>,
    options: Vec<String>,
    owner_value: String,
    binding_writer: Rc<RefCell<Option<Rc<dyn Fn(String, &mut App)>>>>,
}

fn with_current_writer<T: Clone, E, R>(
    writer: &RefCell<T>,
    event: E,
    apply: impl FnOnce(T, E) -> R,
) -> R {
    let writer = writer.borrow().clone();
    apply(writer, event)
}

/// A controlled searchable single-selection ComboBox.
pub struct ComboBoxView {
    label: SharedString,
    placeholder_message_id: Option<SharedString>,
    selection: Binding<String>,
    options: Vec<String>,
    id: Option<Identity>,
    disabled: bool,
    search_placeholder: Option<SharedString>,
    search_placeholder_message_id: Option<SharedString>,
    menu_width: Option<f32>,
    accessibility_label: Option<ComboBoxAccessibilityLabel>,
    appearance: bool,
}

impl ComboBoxView {
    pub fn new<I, S>(label: impl Into<SharedString>, selection: Binding<String>, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            label: label.into(),
            placeholder_message_id: None,
            selection,
            options: collect_unique_options(options),
            id: None,
            disabled: false,
            search_placeholder: None,
            search_placeholder_message_id: None,
            menu_width: None,
            accessibility_label: None,
            appearance: true,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "ComboBox IDs must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = Some(placeholder.into());
        self
    }

    /// Resolves the searchable menu placeholder from the active catalog.
    pub fn localized_search_placeholder(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "ComboBox search placeholder localization key must not be blank"
        );
        self.search_placeholder_message_id = Some(id);
        self
    }

    /// Sets the searchable menu width in logical pixels.
    pub fn menu_width(mut self, width: f32) -> Self {
        assert!(
            width.is_finite() && width > 0.0,
            "ComboBox menu width must be positive and finite"
        );
        self.menu_width = Some(width);
        self
    }

    /// Resolves the visible empty-selection placeholder from the active catalog.
    /// The constructor label remains the fallback when the key is unavailable.
    pub fn localized_placeholder(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "ComboBox placeholder localization key must not be blank"
        );
        self.placeholder_message_id = Some(id);
        self
    }

    fn resolved_placeholder(&self, localized_message: impl FnOnce(&str) -> String) -> SharedString {
        match &self.placeholder_message_id {
            Some(id) => resolved_accessibility_label(
                id.as_ref(),
                localized_message(id.as_ref()),
                &self.label,
            ),
            None => self.label.clone(),
        }
    }

    fn resolved_search_placeholder(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> Option<SharedString> {
        match &self.search_placeholder_message_id {
            Some(id) => Some(resolved_accessibility_label(
                id.as_ref(),
                localized_message(id.as_ref()),
                self.search_placeholder.as_ref().unwrap_or(&self.label),
            )),
            None => self.search_placeholder.clone(),
        }
    }

    /// Sets an accessible name independently from the visible placeholder.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "ComboBox accessible name must not be blank"
        );
        self.accessibility_label = Some(ComboBoxAccessibilityLabel::Explicit(label));
        self
    }

    /// Resolves the ComboBox accessible name from the active RustUI message catalog.
    /// The constructor label remains the visible placeholder and is used as fallback.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "ComboBox accessibility localization keys must be nonempty"
        );
        let fallback = self.accessibility_label.as_ref().map_or_else(
            || self.label.clone(),
            |label| match label {
                ComboBoxAccessibilityLabel::Explicit(label) => label.clone(),
                ComboBoxAccessibilityLabel::Message { fallback, .. } => fallback.clone(),
            },
        );
        self.accessibility_label = Some(ComboBoxAccessibilityLabel::Message { id, fallback });
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
            .unwrap_or_else(|| allocate_implicit(Identity::from("combo-box")))
    }
}

#[allow(non_snake_case)]
pub fn ComboBox<I, S>(
    label: impl Into<SharedString>,
    selection: Binding<String>,
    options: I,
) -> ComboBoxView
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ComboBoxView::new(label, selection, options)
}

impl View for ComboBoxView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity(context);
        let options = self.options.clone();
        let initial_value = self.selection.value();
        let initial_index = option_index(&options, &initial_value);
        let binding_writer = Rc::new(RefCell::new(None));
        let (retained, created) =
            context.retained_entity(identity.clone(), move |window, cx| RetainedComboBox {
                select: cx.new(|cx| {
                    SelectState::new(
                        SearchableVec::new(options.clone()),
                        initial_index,
                        window,
                        cx,
                    )
                    .searchable(true)
                }),
                options,
                owner_value: initial_value,
                binding_writer: binding_writer.clone(),
            });

        let (select, retained_options, retained_owner_value, binding_writer) = {
            let retained = retained.read(context.app());
            (
                retained.select.clone(),
                retained.options.clone(),
                retained.owner_value.clone(),
                retained.binding_writer.clone(),
            )
        };
        refresh_binding_writer(
            &binding_writer,
            context.binding_writer(self.selection.clone()),
        );

        let options_changed = retained_options != self.options;
        if options_changed {
            let options = self.options.clone();
            context.update_entity(&select, |state, window, cx| {
                state.set_items(SearchableVec::new(options.clone()), window, cx);
            });
            context.update_entity(&retained, move |retained, _window, _cx| {
                retained.options = options;
            });
        }

        let current = self.selection.value();
        if combo_box_needs_reconciliation(options_changed, &retained_owner_value, &current) {
            let selection = current.clone();
            context.update_entity(&select, move |state, window, cx| {
                state.set_selected_value(&selection, window, cx);
            });
            context.update_entity(&retained, move |retained, _window, _cx| {
                retained.owner_value = current;
            });
        }

        if created {
            let binding_writer = binding_writer.clone();
            let retained = retained.clone();
            context.subscribe_retained(
                &identity,
                &select,
                move |_state, event: &SelectEvent<SearchableVec<String>>, cx| {
                    let SelectEvent::Confirm(value) = event;
                    if let Some(value) = value {
                        let confirmed_value = value.clone();
                        retained.update(cx, |retained, _cx| {
                            record_confirmed_selection(&mut retained.owner_value, &confirmed_value);
                        });
                        with_current_writer(&binding_writer, value.clone(), |writer, value| {
                            if let Some(write) = writer {
                                write(value, cx);
                            }
                        });
                    }
                },
            );
        }

        let placeholder = self.resolved_placeholder(|id| context.localized_message(id));
        let accessible_name = match &self.accessibility_label {
            Some(ComboBoxAccessibilityLabel::Explicit(label)) => label.clone(),
            Some(ComboBoxAccessibilityLabel::Message { id, fallback }) => {
                resolved_accessibility_label(
                    id.as_ref(),
                    context.localized_message(id.as_ref()),
                    fallback,
                )
            }
            None => placeholder.clone(),
        };
        assert!(
            !accessible_name.trim().is_empty(),
            "ComboBox requires a nonblank visible label or accessible name"
        );
        let mut combo_box = GpuiSelect::new(&select)
            .id(identity.to_string())
            .placeholder(placeholder)
            .accessibility_label(accessible_name)
            .disabled(self.disabled)
            .appearance(self.appearance);
        if let Some(placeholder) =
            self.resolved_search_placeholder(|id| context.localized_message(id))
        {
            combo_box = combo_box.search_placeholder(placeholder);
        }
        if let Some(width) = self.menu_width {
            combo_box = combo_box.menu_width(px(width));
        }
        if !self.appearance {
            return combo_box.into_any_element();
        }

        // Keep GPUI Select's searchable menu and focus behavior. Theme only
        // the 16 px arrow affordance described by the component record.
        let style = required_style(context, "combo_box", "body", &selector(context));
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
            .child(combo_box.icon(Icon::empty()))
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
        "ComboBox options must not be blank"
    );
    assert!(
        options.iter().all(|option| seen.insert(option.as_str())),
        "ComboBox options must be unique"
    );
    options
}

fn option_index(options: &[String], selection: &str) -> Option<IndexPath> {
    options
        .iter()
        .position(|option| option == selection)
        .map(IndexPath::new)
}

fn combo_box_needs_reconciliation(
    options_changed: bool,
    previous_owner_value: &str,
    current_owner_value: &str,
) -> bool {
    options_changed || previous_owner_value != current_owner_value
}

fn record_confirmed_selection(owner_value: &mut String, confirmed_value: &str) {
    *owner_value = confirmed_value.to_owned();
}

fn refresh_binding_writer(
    binding_writer: &RefCell<Option<Rc<dyn Fn(String, &mut App)>>>,
    writer: Rc<dyn Fn(String, &mut App)>,
) {
    *binding_writer.borrow_mut() = Some(writer);
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
    fn explicit_accessible_name_allows_blank_visible_label() {
        let combo_box = ComboBox(" \t ", binding(), ["One"]).accessibility_label("Language");

        assert_eq!(combo_box.label.trim(), "");
        assert!(matches!(
            combo_box.accessibility_label,
            Some(ComboBoxAccessibilityLabel::Explicit(ref label)) if label.as_ref() == "Language"
        ));
    }

    #[test]
    fn preserves_nonblank_accessible_name_verbatim() {
        let combo_box = ComboBox("  Choose a language  ", binding(), ["Rust"]);

        assert_eq!(combo_box.label.as_ref(), "  Choose a language  ");
    }

    #[test]
    fn localized_placeholder_resolves_and_preserves_constructor_fallback() {
        let combo_box = ComboBox("Choose a language", binding(), ["Rust"])
            .localized_placeholder("combo_box.language");
        assert_eq!(
            combo_box
                .resolved_placeholder(|_| "Choisir une langue".into())
                .as_ref(),
            "Choisir une langue"
        );
        assert_eq!(
            combo_box.resolved_placeholder(|id| id.into()).as_ref(),
            "Choose a language"
        );
    }

    #[test]
    fn localized_search_placeholder_resolves_and_preserves_explicit_fallback() {
        let combo_box = ComboBox("Rendering engine", binding(), ["GPUI"])
            .search_placeholder("Find an engine")
            .localized_search_placeholder("combo_box.search");
        assert_eq!(
            combo_box
                .resolved_search_placeholder(|_| "Rechercher un moteur".into())
                .as_deref(),
            Some("Rechercher un moteur")
        );
        assert_eq!(
            combo_box
                .resolved_search_placeholder(|id| id.into())
                .as_deref(),
            Some("Find an engine")
        );
    }

    #[test]
    fn menu_width_is_explicit_and_validated() {
        assert_eq!(
            ComboBox("Rendering engine", binding(), ["GPUI"])
                .menu_width(220.0)
                .menu_width,
            Some(220.0)
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ComboBox("Rendering engine", binding(), ["GPUI"]).menu_width(f32::NAN)
            }))
            .is_err()
        );
    }

    #[test]
    #[should_panic(expected = "ComboBox options must not be blank")]
    fn rejects_blank_accessible_option_names() {
        ComboBox("Choose a language", binding(), ["Rust", " \t "]);
    }

    #[test]
    fn localized_accessible_name_uses_constructor_label_as_fallback() {
        let combo_box = ComboBox("Choose a language", binding(), ["Rust"])
            .localized_accessibility_label("combo_box.language");
        let fallback: SharedString = "Choose a language".into();

        assert!(matches!(
            combo_box.accessibility_label,
            Some(ComboBoxAccessibilityLabel::Message { ref id, ref fallback })
                if id.as_ref() == "combo_box.language"
                    && fallback.as_ref() == "Choose a language"
        ));
        assert_eq!(
            resolved_accessibility_label(
                "combo_box.language",
                "Choisir une langue".into(),
                &fallback,
            )
            .as_ref(),
            "Choisir une langue"
        );
        assert_eq!(
            resolved_accessibility_label("combo_box.language", " \t".into(), &fallback).as_ref(),
            "Choose a language"
        );
        assert_eq!(
            resolved_accessibility_label(
                "combo_box.language",
                "combo_box.language".into(),
                &fallback,
            )
            .as_ref(),
            "Choose a language"
        );

        let explicitly_named = ComboBox("Choose a language", binding(), ["Rust"])
            .accessibility_label("Language")
            .localized_accessibility_label("combo_box.language");
        assert!(matches!(
            explicitly_named.accessibility_label,
            Some(ComboBoxAccessibilityLabel::Message { ref fallback, .. })
                if fallback.as_ref() == "Language"
        ));
    }

    #[test]
    #[should_panic(expected = "ComboBox accessibility localization keys must be nonempty")]
    fn localized_accessible_name_rejects_blank_message_id() {
        ComboBox("Choose a language", binding(), ["Rust"]).localized_accessibility_label(" \t ");
    }

    #[test]
    fn padded_localization_key_echo_uses_constructor_label_fallback() {
        let fallback: SharedString = "Choose a language".into();

        assert_eq!(
            resolved_accessibility_label(
                "combo_box.language",
                " combo_box.language \t".into(),
                &fallback,
            ),
            fallback
        );
    }

    #[test]
    fn unchanged_unmatched_owner_value_does_not_reset_search_state() {
        assert!(!combo_box_needs_reconciliation(false, "", ""));
    }

    #[test]
    fn changed_owner_value_and_options_reconcile_retained_selection() {
        assert!(combo_box_needs_reconciliation(false, "", "Skia"));
        assert!(combo_box_needs_reconciliation(true, "Skia", "Skia"));
    }

    #[test]
    fn rejected_confirmation_reconciles_to_controlled_owner_value() {
        let mut retained_owner_value = String::from("Rust");

        record_confirmed_selection(&mut retained_owner_value, "Swift");

        assert!(combo_box_needs_reconciliation(
            false,
            &retained_owner_value,
            "Rust"
        ));
    }
}

#[test]
fn same_identity_rerender_replaces_the_confirm_binding_writer() {
    let binding_writer = RefCell::new(None);
    let initial: Rc<dyn Fn(String, &mut App)> = Rc::new(|_, _| {});
    let latest: Rc<dyn Fn(String, &mut App)> = Rc::new(|_, _| {});
    refresh_binding_writer(&binding_writer, initial.clone());
    refresh_binding_writer(&binding_writer, latest.clone());
    assert!(Rc::ptr_eq(
        binding_writer.borrow().as_ref().unwrap(),
        &latest,
    ));
}

#[test]
fn retained_confirmation_releases_writer_slot_before_applying() {
    type TestWriter = Rc<dyn Fn(String)>;
    let current = Rc::new(RefCell::new(Some(Rc::new(|_| {}) as TestWriter)));
    let writer_slot = current.clone();
    *current.borrow_mut() = Some(Rc::new(move |_| {
        *writer_slot.borrow_mut() = Some(Rc::new(|_| {}));
    }));

    with_current_writer(current.as_ref(), "Swift".to_owned(), |writer, value| {
        if let Some(write) = writer {
            write(value);
        }
    });
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
    fn implicit_identity_survives_label_changes_and_distinguishes_siblings() {
        let first = ComboBox("Choice", binding(), ["One"]);
        let second = ComboBox("Choice", binding(), ["One"]);
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first_sibling = first.identity_with(allocate_at_slot(0));
        let second_sibling = second.identity_with(allocate_at_slot(1));
        let first_after_rerender = first.identity_with(allocate_at_slot(0));
        let first_after_locale_change =
            ComboBox("Choix", binding(), ["One"]).identity_with(allocate_at_slot(0));

        assert_ne!(first_sibling, second_sibling);
        assert_eq!(first_sibling, first_after_rerender);
        assert_eq!(first_sibling, first_after_locale_change);
    }
}
