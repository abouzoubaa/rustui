//! Editable token input backed by controlled RustUI bindings.

use gpui::{AnyElement, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::{
    Disableable as _, Icon, IconName, Sizable,
    button::{Button as GpuiButton, ButtonVariants as _},
};
use rustui_core::{Binding, Identity};

use crate::{
    components::{basic::badge::Badge, basic::text::Text},
    view::{EmptyView, RenderContext, View},
};

fn token_after_add(tokens: &[String], draft: &str) -> Option<(Vec<String>, String)> {
    let token = draft.trim();
    if token.is_empty() {
        return None;
    }
    let mut next = tokens.to_vec();
    next.push(token.to_owned());
    Some((next, String::new()))
}

fn remove_matching_token(tokens: &mut Vec<String>, index: usize, expected: &str) -> bool {
    if tokens.get(index).map(String::as_str) != Some(expected) {
        return false;
    }
    tokens.remove(index);
    true
}

fn clear_added_draft(draft: &mut String, added: &str) -> bool {
    if draft != added {
        return false;
    }
    draft.clear();
    true
}

fn resolved_accessible_label(id: &str, localized: String, fallback: String) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.into()
    } else {
        localized.into()
    }
}

fn add_accessible_label(localized: impl FnOnce(&str) -> String) -> SharedString {
    let id = "rustui-token-field-add";
    resolved_accessible_label(id, localized(id), "Add token".to_owned())
}

fn add_button_label(localized: impl FnOnce(&str) -> String) -> SharedString {
    let id = "rustui-token-field-add-button";
    resolved_accessible_label(id, localized(id), "Add".to_owned())
}

fn remove_accessible_label(
    token: &str,
    localized: impl FnOnce(
        &str,
        &crate::localization::MessageArguments,
    ) -> Result<String, crate::localization::InterpolationError>,
) -> SharedString {
    let (id, fallback, arguments) = if token.trim().is_empty() {
        (
            "rustui-token-field-remove-empty",
            "Remove token".to_owned(),
            crate::localization::MessageArguments::new(),
        )
    } else {
        (
            "rustui-token-field-remove",
            format!("Remove {token}"),
            crate::localization::MessageArguments::new().with("token", token),
        )
    };
    resolved_accessible_label(
        id,
        localized(id, &arguments).unwrap_or_else(|_| id.to_owned()),
        fallback,
    )
}

/// A controlled token input with removable badges and an explicit Add button.
///
/// `tokens` stores committed values and `draft` stores the text being entered.
/// Blank drafts are ignored; committed values are trimmed and otherwise kept
/// verbatim, including duplicates.
pub struct TokenFieldView {
    placeholder: SharedString,
    placeholder_message_id: Option<SharedString>,
    tokens: Binding<Vec<String>>,
    draft: Binding<String>,
    id: Option<Identity>,
    disabled: bool,
}

impl TokenFieldView {
    pub fn new(
        placeholder: impl Into<SharedString>,
        tokens: Binding<Vec<String>>,
        draft: Binding<String>,
    ) -> Self {
        Self {
            placeholder: placeholder.into(),
            placeholder_message_id: None,
            tokens,
            draft,
            id: None,
            disabled: false,
        }
    }

    /// Sets the stable identity used by this field and its child controls.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "TokenField IDs must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Resolves the entry placeholder through the active locale catalog.
    pub fn localized_placeholder(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "TokenField placeholder localization key must not be blank"
        );
        self.placeholder_message_id = Some(id);
        self
    }

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| allocate_implicit(Identity::from("token-field")))
    }
}

/// Creates a token field whose values and draft are controlled by bindings.
#[allow(non_snake_case)]
pub fn TokenField(
    placeholder: impl Into<SharedString>,
    tokens: Binding<Vec<String>>,
    draft: Binding<String>,
) -> TokenFieldView {
    TokenFieldView::new(placeholder, tokens, draft)
}

impl View for TokenFieldView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity(context);
        let tokens = self.tokens.value();
        let draft = self.draft.value();
        let add_token =
            context.binding_event_try_updater(self.tokens.clone(), |tokens, draft: String| {
                if let Some((next, _)) = token_after_add(tokens, &draft) {
                    *tokens = next;
                    true
                } else {
                    false
                }
            });
        let read_draft = context.binding_reader(self.draft.clone());
        let clear_draft = context
            .binding_event_updater(self.draft.clone(), |draft, added: String| {
                clear_added_draft(draft, &added)
            });

        let mut field = div()
            .id(identity.to_string())
            .flex()
            .items_center()
            .flex_wrap()
            .gap_2();
        for (index, token) in tokens.iter().enumerate() {
            let remove_id = format!("{identity}-remove-{index}");
            let remove_label = remove_accessible_label(token, |id, arguments| {
                context.localized_message_with_arguments(id, arguments)
            });
            let expected = token.clone();
            let remove_token = context.binding_updater(self.tokens.clone(), move |tokens| {
                remove_matching_token(tokens, index, &expected)
            });
            let remove = GpuiButton::new(remove_id)
                .icon(Icon::new(IconName::Close))
                .xsmall()
                .text()
                .disabled(self.disabled)
                .accessibility_label(remove_label)
                .on_click(move |_, _, cx| remove_token(cx));
            let chip = div()
                .flex()
                .items_center()
                .gap_1()
                .child(Badge(Text(token.clone())).into_element(context))
                .child(remove);
            field = field.child(chip);
        }

        let mut input_view = crate::components::input::text_field::TextField(
            self.placeholder.clone(),
            self.draft.clone(),
        )
        .id(format!("{identity}-input"))
        .disabled(self.disabled);
        if let Some(message_id) = &self.placeholder_message_id {
            input_view = input_view.localized_placeholder(message_id.clone());
        }
        let mut input = input_view.into_element(context);
        // Keep a useful minimum target as chips wrap around the entry field.
        input = div().min_w(gpui::px(120.0)).child(input).into_any_element();
        field = field.child(input);

        let add_enabled = !self.disabled && !draft.trim().is_empty();
        let add = GpuiButton::new(format!("{identity}-add"))
            .child(add_button_label(|id| context.localized_message(id)))
            .disabled(!add_enabled)
            .accessibility_label(add_accessible_label(|id| context.localized_message(id)))
            .on_click(move |_, _, cx| {
                if let Some(draft) = read_draft(cx).filter(|draft| !draft.trim().is_empty()) {
                    if add_token(draft.clone(), cx) {
                        clear_draft(draft, cx);
                    }
                }
            });
        field.child(add).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TokenField, TokenFieldView, add_accessible_label, add_button_label,
        remove_accessible_label, token_after_add,
    };
    use rustui_core::Binding;

    fn binding<T: Clone + 'static>(value: T) -> Binding<T> {
        Binding::new(
            value,
            |value: &T| value.clone(),
            |value, next| *value = next,
        )
    }

    #[test]
    fn token_field_add_trims_text_and_clears_the_draft_without_deduplicating() {
        assert_eq!(
            token_after_add(&["Atlas".into()], "  Studio  "),
            Some((vec!["Atlas".into(), "Studio".into()], String::new()))
        );
        assert_eq!(
            token_after_add(&["Atlas".into()], "Atlas"),
            Some((vec!["Atlas".into(), "Atlas".into()], String::new()))
        );
    }

    #[test]
    fn token_field_ignores_blank_drafts() {
        assert_eq!(token_after_add(&["Atlas".into()], " \t "), None);
    }

    #[test]
    fn token_field_removes_only_the_selected_token_and_ignores_stale_indices() {
        let mut tokens = vec!["Atlas".into(), "Studio".into(), "Notes".into()];
        assert!(!super::remove_matching_token(&mut tokens, 3, "Notes"));
        assert!(super::remove_matching_token(&mut tokens, 1, "Studio"));
        assert_eq!(tokens, vec!["Atlas", "Notes"]);
    }

    #[test]
    fn token_edits_preserve_latest_owner_values_and_ignore_stale_removal() {
        let mut latest: Vec<String> = vec!["Atlas".into(), "Studio".into(), "Notes".into()];
        let tokens = binding(vec!["Atlas".into(), "old token".into()]);
        assert!(!tokens.update_on(&mut latest, |value| {
            super::remove_matching_token(value, 1, "old token")
        }));
        assert!(tokens.update_on(&mut latest, |value| {
            super::remove_matching_token(value, 1, "Studio")
        }));
        assert_eq!(latest, vec!["Atlas", "Notes"]);
        assert!(tokens.update_on(&mut latest, |value| {
            *value = token_after_add(value, " New ").unwrap().0;
            true
        }));
        assert_eq!(latest, vec!["Atlas", "Notes", "New"]);
    }

    #[test]
    fn adding_a_token_does_not_clear_a_newer_owner_draft() {
        let draft = binding(String::from("original"));
        let mut owner = String::from("new draft from token setter");
        assert!(
            !draft.update_on(&mut owner, |value| super::clear_added_draft(
                value, "original"
            ))
        );
        assert_eq!(owner, "new draft from token setter");
        assert!(
            draft.update_on(&mut owner, |value| super::clear_added_draft(
                value,
                "new draft from token setter"
            ))
        );
        assert!(owner.is_empty());
    }

    #[test]
    fn token_field_action_names_use_localized_catalog_messages_with_fallbacks() {
        assert_eq!(
            add_button_label(|id| {
                assert_eq!(id, "rustui-token-field-add-button");
                "Ajouter".to_owned()
            }),
            "Ajouter"
        );
        assert_eq!(add_button_label(|id| id.to_owned()), "Add");
        assert_eq!(
            add_accessible_label(|id| {
                assert_eq!(id, "rustui-token-field-add");
                "Ajouter un jeton".to_owned()
            }),
            "Ajouter un jeton"
        );
        assert_eq!(add_accessible_label(|id| id.to_owned()), "Add token");

        assert_eq!(
            remove_accessible_label("Atlas", |id, arguments| {
                assert_eq!(id, "rustui-token-field-remove");
                crate::localization::interpolate("Supprimer {token}", arguments)
            }),
            "Supprimer Atlas"
        );
        assert_eq!(
            remove_accessible_label("Atlas", |id, _| Ok(id.to_owned())),
            "Remove Atlas"
        );
        assert_eq!(
            remove_accessible_label("", |id, _| {
                assert_eq!(id, "rustui-token-field-remove-empty");
                Ok("Supprimer le jeton".to_owned())
            }),
            "Supprimer le jeton"
        );
    }

    #[test]
    fn public_token_field_accepts_controlled_token_and_draft_bindings() {
        let _field = TokenField(
            "Add recipients",
            binding(vec!["Atlas".into()]),
            binding(String::new()),
        )
        .id("recipients");
    }

    #[test]
    fn implicit_identity_survives_placeholder_localization_and_distinguishes_siblings() {
        let allocate_at_slot = |slot| {
            move |base: rustui_core::Identity| {
                rustui_core::Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
            }
        };
        let field = |placeholder| {
            TokenFieldView::new(
                placeholder,
                binding(Vec::<String>::new()),
                binding(String::new()),
            )
        };

        let original = field("Add recipients").identity_with(allocate_at_slot(0));
        let translated = field("Ajouter des destinataires").identity_with(allocate_at_slot(0));
        let sibling = field("Add recipients").identity_with(allocate_at_slot(1));

        assert_eq!(original, translated);
        assert_ne!(original, sibling);
    }
}
