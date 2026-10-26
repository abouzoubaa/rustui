//! Multiline text editor backed by a retained GPUI Components TextareaState.

use std::{cell::RefCell, rc::Rc};

use gpui::{AnyElement, AppContext as _, EntityInputHandler as _, IntoElement, SharedString, px};
use gpui_component::input::{InputEvent, Textarea, TextareaState};
use rustui_core::{Binding, Identity};

use crate::view::{EmptyView, RenderContext, View};

const DEFAULT_ACCESSIBILITY_LABEL: &str = "Text editor";

type EventWriter<Value, Context> = Rc<dyn Fn(Value, &mut Context)>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum TextEditorLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_message_label(id: &str, localized: String, fallback: &SharedString) -> SharedString {
    if !localized.trim().is_empty() && localized.trim() != id.trim() {
        SharedString::from(localized)
    } else {
        fallback.clone()
    }
}

fn replace_event_writer<Value, Context>(
    slot: &RefCell<Option<EventWriter<Value, Context>>>,
    writer: EventWriter<Value, Context>,
) {
    *slot.borrow_mut() = Some(writer);
}

fn write_with_current_writer<Value, Context>(
    slot: &RefCell<Option<EventWriter<Value, Context>>>,
    value: Value,
    context: &mut Context,
) {
    let writer = slot.borrow().clone();
    if let Some(writer) = writer {
        writer(value, context);
    }
}

struct RetainedTextEditor {
    state: gpui::Entity<TextareaState>,
    owner_value: String,
    had_marked_text: bool,
    writer: Rc<RefCell<Option<EventWriter<String, gpui::App>>>>,
}

#[derive(Debug, PartialEq, Eq)]
struct OwnerSyncPlan {
    update_editor_value: bool,
    update_owner_snapshot: bool,
    update_marked_text_snapshot: bool,
}

fn owner_sync_plan(
    current_owner_value: &str,
    previous_owner_value: &str,
    retained_value: &str,
    had_marked_text: bool,
    has_marked_text: bool,
) -> OwnerSyncPlan {
    let owner_changed = current_owner_value != previous_owner_value;
    OwnerSyncPlan {
        update_editor_value: if owner_changed {
            current_owner_value != retained_value
        } else {
            had_marked_text && !has_marked_text && retained_value != current_owner_value
        },
        update_owner_snapshot: owner_changed,
        update_marked_text_snapshot: had_marked_text != has_marked_text,
    }
}

fn placeholder_changed(retained_placeholder: &str, requested_placeholder: &str) -> bool {
    retained_placeholder != requested_placeholder
}

/// A controlled multiline text editor whose editing state survives body rebuilds.
pub struct TextEditorView {
    placeholder: SharedString,
    placeholder_message_id: Option<SharedString>,
    value: Binding<String>,
    id: Option<Identity>,
    height: f32,
    disabled: bool,
    readonly: bool,
    accessibility_label: Option<TextEditorLabel>,
    tab_index: isize,
}

impl TextEditorView {
    pub fn new(placeholder: impl Into<SharedString>, value: Binding<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            placeholder_message_id: None,
            value,
            id: None,
            height: 120.0,
            disabled: false,
            readonly: false,
            accessibility_label: None,
            tab_index: 0,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "TextEditor IDs must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        assert!(height.is_finite(), "TextEditor height must be finite");
        self.height = height.max(40.0);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Resolve the visible placeholder through the active RustUI catalog.
    /// The constructor value remains the fallback when the key is missing.
    pub fn localized_placeholder(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "TextEditor placeholder localization key must not be blank"
        );
        self.placeholder_message_id = Some(id);
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "TextEditor accessibility label must not be empty"
        );
        self.accessibility_label = Some(TextEditorLabel::Literal(label));
        self
    }

    /// Resolve the editor's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let message_id = message_id.into();
        assert!(
            !message_id.trim().is_empty(),
            "TextEditor accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(TextEditorLabel::Literal(label))
            | Some(TextEditorLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.default_accessibility_label(),
        };
        self.accessibility_label = Some(TextEditorLabel::Message {
            id: message_id,
            fallback,
        });
        self
    }

    fn default_accessibility_label(&self) -> SharedString {
        if self.placeholder.trim().is_empty() {
            DEFAULT_ACCESSIBILITY_LABEL.into()
        } else {
            self.placeholder.clone()
        }
    }

    fn resolved_placeholder(&self, localized_message: impl FnOnce(&str) -> String) -> SharedString {
        match &self.placeholder_message_id {
            Some(id) => resolved_message_label(
                id.as_ref(),
                localized_message(id.as_ref()),
                &self.placeholder,
            ),
            None => self.placeholder.clone(),
        }
    }

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        match &self.accessibility_label {
            Some(TextEditorLabel::Literal(label)) => label.clone(),
            Some(TextEditorLabel::Message { id, fallback }) => {
                resolved_message_label(id.as_ref(), localized_message(id.as_ref()), fallback)
            }
            None => {
                let placeholder = self.resolved_placeholder(localized_message);
                if placeholder.trim().is_empty() {
                    DEFAULT_ACCESSIBILITY_LABEL.into()
                } else {
                    placeholder
                }
            }
        }
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        if let Some(id) = &self.id {
            return id.clone();
        }

        // The placeholder may change with locale without changing the editor.
        allocate_implicit(Identity::from("text-editor"))
    }
}

/// Creates a controlled multiline text editor.
#[allow(non_snake_case)]
pub fn TextEditor(placeholder: impl Into<SharedString>, value: Binding<String>) -> TextEditorView {
    TextEditorView::new(placeholder, value)
}

impl View for TextEditorView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity(context);
        let initial_value = self.value.value();
        let placeholder = self.resolved_placeholder(|id| context.localized_message(id));
        let initial_placeholder = placeholder.clone();
        let (retained, created) =
            context.retained_entity(identity.clone(), move |window, cx| RetainedTextEditor {
                state: cx.new(|cx| {
                    TextareaState::new(window, cx)
                        .placeholder(initial_placeholder)
                        .default_value(initial_value.clone())
                }),
                owner_value: initial_value,
                had_marked_text: false,
                writer: Rc::new(RefCell::new(None)),
            });
        let state = retained.read(context.app()).state.clone();
        let retained_placeholder = state
            .read(context.app())
            .presentation()
            .placeholder()
            .to_string();
        if placeholder_changed(&retained_placeholder, &placeholder) {
            context.update_entity(&state, move |state, window, cx| {
                state.set_placeholder(placeholder, window, cx);
            });
        }

        let current_value = self.value.value();
        let (retained_value, has_marked_text) =
            context.update_entity(&state, |state, window, cx| {
                (
                    state.value().to_string(),
                    state.marked_text_range(window, cx).is_some(),
                )
            });
        let previous_owner_value = retained.read(context.app()).owner_value.clone();
        let had_marked_text = retained.read(context.app()).had_marked_text;
        let sync_plan = owner_sync_plan(
            &current_value,
            &previous_owner_value,
            &retained_value,
            had_marked_text,
            has_marked_text,
        );
        if sync_plan.update_editor_value {
            let value_to_sync = current_value.clone();
            context.update_entity(&state, move |state, window, cx| {
                state.set_value(value_to_sync, window, cx);
            });
        }
        if sync_plan.update_owner_snapshot {
            context.update_entity(&retained, move |retained, _, _| {
                retained.owner_value = current_value;
            });
        }
        if sync_plan.update_marked_text_snapshot {
            context.update_entity(&retained, move |retained, _, _| {
                retained.had_marked_text = has_marked_text;
            });
        }
        let writer = retained.read(context.app()).writer.clone();
        replace_event_writer(&writer, context.binding_writer(self.value.clone()));

        if created {
            let current_writer = writer.clone();
            context.subscribe_retained(&identity, &state, move |state, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    write_with_current_writer(
                        &current_writer,
                        state.read(cx).value().to_string(),
                        cx,
                    );
                }
            });
        }

        let mut editor = Textarea::new(&state)
            .h(px(self.height))
            .disabled(self.disabled)
            .readonly(self.readonly)
            .tab_index(self.tab_index);
        editor = editor
            .aria_label(self.resolved_accessibility_label(|id| context.localized_message(id)));
        editor.into_any_element()
    }
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
    fn height_rejects_non_finite_values_and_keeps_the_finite_minimum() {
        for height in [f32::NAN, f32::NEG_INFINITY, f32::INFINITY] {
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    TextEditor("Notes", binding()).height(height)
                }))
                .is_err()
            );
        }

        assert_eq!(TextEditor("Notes", binding()).height(0.0).height, 40.0);
        assert_eq!(TextEditor("Notes", binding()).height(64.0).height, 64.0);
    }

    #[test]
    fn changed_placeholder_is_refreshed_in_retained_textarea_state() {
        assert!(placeholder_changed("Write a note", "Meeting notes"));
        assert!(!placeholder_changed("Meeting notes", "Meeting notes"));
    }

    #[test]
    fn owner_sync_preserves_preedit_and_advances_matching_owner_snapshot() {
        assert_eq!(
            owner_sync_plan("draft", "draft", "draftに", false, true),
            OwnerSyncPlan {
                update_editor_value: false,
                update_owner_snapshot: false,
                update_marked_text_snapshot: true,
            }
        );
        assert_eq!(
            owner_sync_plan("external", "draft", "draftに", true, true),
            OwnerSyncPlan {
                update_editor_value: true,
                update_owner_snapshot: true,
                update_marked_text_snapshot: false,
            }
        );
        assert_eq!(
            owner_sync_plan("external", "draft", "external", false, false),
            OwnerSyncPlan {
                update_editor_value: false,
                update_owner_snapshot: true,
                update_marked_text_snapshot: false,
            }
        );

        // Once the owner snapshot advances, later local edits stay authoritative.
        assert_eq!(
            owner_sync_plan("external", "external", "external edit", false, false),
            OwnerSyncPlan {
                update_editor_value: false,
                update_owner_snapshot: false,
                update_marked_text_snapshot: false,
            }
        );

        // When IME clears a canceled composition, restore the unchanged owner
        // value so preedit text cannot remain as the retained editor value.
        assert_eq!(
            owner_sync_plan("draft", "draft", "draftに", true, false),
            OwnerSyncPlan {
                update_editor_value: true,
                update_owner_snapshot: false,
                update_marked_text_snapshot: true,
            }
        );

        // Ordinary edits do not get reverted when their input event has not
        // propagated the latest owner value yet.
        assert_eq!(
            owner_sync_plan("draft", "draft", "draft edit", false, false),
            OwnerSyncPlan {
                update_editor_value: false,
                update_owner_snapshot: false,
                update_marked_text_snapshot: false,
            }
        );
    }

    #[test]
    fn accessible_name_remains_optional_and_preserves_nonblank_text() {
        assert!(TextEditor("Notes", binding()).accessibility_label.is_none());

        let editor = TextEditor("Notes", binding()).accessibility_label(" Notes for today ");
        assert_eq!(
            editor
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            " Notes for today "
        );
    }

    #[test]
    fn localized_placeholder_updates_visible_text_and_default_accessible_name() {
        let editor = TextEditor("Notes", binding()).localized_placeholder("editor.notes");
        assert_eq!(
            editor
                .resolved_placeholder(|_| "ملاحظات".to_owned())
                .as_ref(),
            "ملاحظات"
        );
        assert_eq!(
            editor
                .resolved_accessibility_label(|_| "ملاحظات".to_owned())
                .as_ref(),
            "ملاحظات"
        );
        assert_eq!(
            editor.resolved_placeholder(|id| id.to_owned()).as_ref(),
            "Notes"
        );
        assert!(placeholder_changed("Notes", "ملاحظات"));
    }

    #[test]
    fn localized_accessible_name_resolves_translation_and_placeholder_fallback() {
        let editor = TextEditor("Notes", binding()).localized_accessibility_label("editor.notes");

        assert!(matches!(
            &editor.accessibility_label,
            Some(TextEditorLabel::Message { id, fallback })
                if id.as_ref() == "editor.notes" && fallback.as_ref() == "Notes"
        ));
        assert_eq!(
            editor
                .resolved_accessibility_label(|_| "Notes en français".to_owned())
                .as_ref(),
            "Notes en français"
        );
        assert_eq!(
            editor
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            "Notes"
        );
        assert_eq!(
            editor
                .resolved_accessibility_label(|_| " \t".to_owned())
                .as_ref(),
            "Notes"
        );
    }

    #[test]
    fn padded_localization_key_echo_uses_accessible_name_fallback() {
        let editor = TextEditor("Notes", binding()).localized_accessibility_label("editor.notes");

        assert_eq!(
            editor
                .resolved_accessibility_label(|_| " editor.notes \t".to_owned())
                .as_ref(),
            "Notes"
        );
    }

    #[test]
    fn localized_accessible_name_keeps_explicit_literal_fallback() {
        let editor = TextEditor("Notes", binding())
            .accessibility_label("Meeting notes")
            .localized_accessibility_label("editor.meeting_notes");

        assert_eq!(
            editor
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            "Meeting notes"
        );
        assert_eq!(
            editor
                .resolved_accessibility_label(|_| "Notes de réunion".to_owned())
                .as_ref(),
            "Notes de réunion"
        );
    }

    #[test]
    fn localized_accessible_name_defaults_when_placeholder_is_blank() {
        let default_editor = TextEditor(" \t", binding());
        assert_eq!(
            default_editor
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            DEFAULT_ACCESSIBILITY_LABEL
        );

        let editor = default_editor.localized_accessibility_label("editor.default");

        assert_eq!(
            editor
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            DEFAULT_ACCESSIBILITY_LABEL
        );
    }

    #[test]
    #[should_panic(expected = "TextEditor accessibility localization key must not be blank")]
    fn rejects_blank_accessibility_localization_key() {
        TextEditor("Notes", binding()).localized_accessibility_label(" \t ");
    }

    #[test]
    fn explicit_retained_identity_is_kept_on_the_view() {
        let identity = Identity::from(" editor-notes ");

        assert_eq!(
            TextEditor("Notes", binding()).id(identity.clone()).id,
            Some(identity)
        );
    }

    #[test]
    fn implicit_identity_survives_placeholder_localization_and_separates_siblings() {
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };
        let original = TextEditor("Notes", binding()).identity_with(allocate_at_slot(0));
        let translated =
            TextEditor("Notes de réunion", binding()).identity_with(allocate_at_slot(0));
        let sibling = TextEditor("Notes", binding()).identity_with(allocate_at_slot(1));

        assert_eq!(original, translated);
        assert_ne!(original, sibling);
    }

    #[test]
    #[should_panic(expected = "TextEditor IDs must be nonempty")]
    fn rejects_empty_explicit_identity() {
        TextEditor("Notes", binding()).id("");
    }

    #[test]
    #[should_panic(expected = "TextEditor IDs must be nonempty")]
    fn rejects_whitespace_only_explicit_identity() {
        TextEditor("Notes", binding()).id(" \t ");
    }

    #[test]
    #[should_panic(expected = "TextEditor accessibility label must not be empty")]
    fn rejects_whitespace_only_accessible_name() {
        TextEditor("Notes", binding()).accessibility_label("\t ");
    }

    #[test]
    fn retained_events_use_the_most_recent_owner_binding() {
        struct Owner {
            old_value: String,
            current_value: String,
        }

        let old_binding = Binding::new_with::<Owner, &'static str>(
            String::new(),
            "old",
            |owner, field| match *field {
                "old" => owner.old_value.clone(),
                _ => owner.current_value.clone(),
            },
            |owner, field, next| {
                match *field {
                    "old" => owner.old_value = next,
                    _ => owner.current_value = next,
                }
                true
            },
        );
        let current_binding = Binding::new_with::<Owner, &'static str>(
            String::new(),
            "current",
            |owner, field| match *field {
                "old" => owner.old_value.clone(),
                _ => owner.current_value.clone(),
            },
            |owner, field, next| {
                match *field {
                    "old" => owner.old_value = next,
                    _ => owner.current_value = next,
                }
                true
            },
        );
        let writer = RefCell::new(None);
        let mut owner = Owner {
            old_value: String::new(),
            current_value: String::new(),
        };
        replace_event_writer(
            &writer,
            Rc::new(move |value, owner: &mut Owner| {
                old_binding.write_to(owner, value);
            }),
        );
        replace_event_writer(
            &writer,
            Rc::new(move |value, owner: &mut Owner| {
                current_binding.write_to(owner, value);
            }),
        );

        write_with_current_writer(&writer, "edited".to_owned(), &mut owner);

        assert_eq!(owner.old_value, "");
        assert_eq!(owner.current_value, "edited");
    }
}
