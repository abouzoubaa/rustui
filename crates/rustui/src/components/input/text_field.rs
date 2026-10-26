//! Single-line text input backed by a retained GPUI Components InputState.

use gpui::{
    AnyElement, AppContext, Entity, EntityInputHandler, IntoElement, ParentElement as _,
    SharedString, Styled, div,
};
use gpui_component::{
    Disableable as _, Icon, IconName, Sizable,
    button::{Button as GpuiButton, ButtonVariants as _},
    input::{Input, InputContentType, InputEvent, InputState},
};
use rustui_core::{Binding, Identity};
use std::{cell::RefCell, ops::Range, rc::Rc};

use crate::{
    components::theme::{required_style_with_artwork, selector, styled_element},
    theme::{AppearanceMode, ThemeProfile},
    view::{EmptyView, RenderContext, View},
};

const DEFAULT_ACCESSIBILITY_LABEL: &str = "Text field";
const DEFAULT_PASSWORD_VISIBILITY_ACCESSIBILITY_LABEL: &str = "Toggle password visibility";
const CLEAR_BUTTON_TAB_STOP: bool = true;

#[derive(Clone, Debug, PartialEq, Eq)]
enum TextFieldAccessibilityLabel {
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextFieldKind {
    #[default]
    Plain,
    Secure,
    Search,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFieldSize {
    #[default]
    Small,
    Large,
}

struct RetainedTextField {
    state: Entity<InputState>,
    owner_value: String,
    last_selection: Option<Range<usize>>,
    selection_before_preedit: Option<Range<usize>>,
    secure: bool,
    writer: Rc<RefCell<Option<EventWriter<String, gpui::App>>>>,
}

type EventWriter<Value, Context> = Rc<dyn Fn(Value, &mut Context)>;

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

impl TextFieldSize {
    fn selector_value(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Large => "large",
        }
    }
}

/// A controlled single-line text field whose editing state survives body rebuilds.
pub struct TextFieldView {
    placeholder: SharedString,
    placeholder_message_id: Option<SharedString>,
    value: Binding<String>,
    id: Option<Identity>,
    kind: TextFieldKind,
    size: TextFieldSize,
    disabled: bool,
    readonly: bool,
    cleanable: bool,
    accessibility_label: Option<TextFieldAccessibilityLabel>,
    clear_button_accessibility_label: Option<TextFieldAccessibilityLabel>,
    password_visibility_accessibility_label: Option<TextFieldAccessibilityLabel>,
    tab_index: isize,
}

impl TextFieldView {
    pub fn new(placeholder: impl Into<SharedString>, value: Binding<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            placeholder_message_id: None,
            value,
            id: None,
            kind: TextFieldKind::Plain,
            size: TextFieldSize::Small,
            disabled: false,
            readonly: false,
            cleanable: false,
            accessibility_label: None,
            clear_button_accessibility_label: None,
            password_visibility_accessibility_label: None,
            tab_index: 0,
        }
    }

    /// Resolves the visible placeholder from the active RustUI catalog.
    /// The constructor's placeholder remains the fallback if the key is missing.
    pub fn localized_placeholder(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "TextField placeholder localization key must not be blank"
        );
        self.placeholder_message_id = Some(id);
        self
    }

    pub(crate) fn with_kind(mut self, kind: TextFieldKind) -> Self {
        self.kind = kind;
        if kind == TextFieldKind::Search {
            self.cleanable = true;
        }
        self
    }

    pub fn size(mut self, size: TextFieldSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the stable identity used to retain cursor, selection, undo, focus, and IME state.
    /// Without an explicit identity, fields use their render owner and sibling position;
    /// supply an ID when fields may be reordered or conditionally replaced.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "TextField IDs must be nonempty"
        );
        self.id = Some(id);
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

    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "TextField accessibility label must not be empty"
        );
        self.accessibility_label = Some(TextFieldAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the field's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "TextField accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(TextFieldAccessibilityLabel::Literal(label))
            | Some(TextFieldAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.default_accessibility_label(),
        };
        self.accessibility_label = Some(TextFieldAccessibilityLabel::Message { id, fallback });
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
            Some(TextFieldAccessibilityLabel::Literal(label)) => label.clone(),
            Some(TextFieldAccessibilityLabel::Message { id, fallback }) => {
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

    /// Sets a localized accessible name for the optional clear button.
    pub fn clear_button_accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "TextField clear button accessibility label must not be blank"
        );
        self.clear_button_accessibility_label = Some(TextFieldAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the clear button's accessible name from the active RustUI catalog.
    pub fn localized_clear_button_accessibility_label(
        mut self,
        message_id: impl Into<SharedString>,
    ) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "TextField clear button accessibility localization key must not be blank"
        );
        let fallback = match &self.clear_button_accessibility_label {
            Some(TextFieldAccessibilityLabel::Literal(label))
            | Some(TextFieldAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => clear_accessibility_label(&self.placeholder).into(),
        };
        self.clear_button_accessibility_label =
            Some(TextFieldAccessibilityLabel::Message { id, fallback });
        self
    }

    fn resolved_clear_button_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        match &self.clear_button_accessibility_label {
            Some(TextFieldAccessibilityLabel::Literal(label)) => label.clone(),
            Some(TextFieldAccessibilityLabel::Message { id, fallback }) => {
                resolved_message_label(id.as_ref(), localized_message(id.as_ref()), fallback)
            }
            None => clear_accessibility_label(&self.placeholder).into(),
        }
    }

    /// Sets a literal accessible name for a SecureField's visibility control.
    pub fn password_visibility_accessibility_label(
        mut self,
        label: impl Into<SharedString>,
    ) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "TextField password visibility accessibility label must not be blank"
        );
        self.password_visibility_accessibility_label =
            Some(TextFieldAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the visibility control's accessible name from the active RustUI catalog.
    pub fn localized_password_visibility_accessibility_label(
        mut self,
        message_id: impl Into<SharedString>,
    ) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "TextField password visibility accessibility localization key must not be blank"
        );
        let fallback = match &self.password_visibility_accessibility_label {
            Some(TextFieldAccessibilityLabel::Literal(label))
            | Some(TextFieldAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => DEFAULT_PASSWORD_VISIBILITY_ACCESSIBILITY_LABEL.into(),
        };
        self.password_visibility_accessibility_label =
            Some(TextFieldAccessibilityLabel::Message { id, fallback });
        self
    }

    fn resolved_password_visibility_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        match &self.password_visibility_accessibility_label {
            Some(TextFieldAccessibilityLabel::Literal(label)) => label.clone(),
            Some(TextFieldAccessibilityLabel::Message { id, fallback }) => {
                resolved_message_label(id.as_ref(), localized_message(id.as_ref()), fallback)
            }
            None => DEFAULT_PASSWORD_VISIBILITY_ACCESSIBILITY_LABEL.into(),
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

        let kind = match self.kind {
            TextFieldKind::Plain => "text-field",
            TextFieldKind::Secure => "secure-field",
            TextFieldKind::Search => "search-field",
        };
        allocate_implicit(Identity::from(kind))
    }
}

fn should_sync_owner_value(
    current_owner_value: &str,
    previous_owner_value: &str,
    retained_value: &str,
    marked_range: Option<Range<usize>>,
) -> bool {
    retained_value != current_owner_value
        && !(marked_range.is_some() && current_owner_value == previous_owner_value)
}

fn placeholder_changed(retained_placeholder: &str, requested_placeholder: &str) -> bool {
    retained_placeholder != requested_placeholder
}

fn reconcile_masked_state(current_masked: bool, was_secure: bool, secure: bool) -> bool {
    if was_secure == secure {
        current_masked
    } else {
        secure
    }
}

fn visibility_control_is_toggled(masked: bool) -> bool {
    !masked
}

fn clear_accessibility_label(placeholder: &str) -> String {
    if placeholder.trim().is_empty() {
        "Clear text field".to_owned()
    } else {
        format!("Clear {placeholder}")
    }
}

fn disabled_text_color(profile: &ThemeProfile, mode: AppearanceMode) -> gpui::Hsla {
    gpui_component::theme::try_parse_color(&profile.colors(Some(mode)).disabled_text)
        .expect("validated RustUI text field disabled text color")
}

/// Creates a controlled single-line text field.
#[allow(non_snake_case)]
pub fn TextField(placeholder: impl Into<SharedString>, value: Binding<String>) -> TextFieldView {
    TextFieldView::new(placeholder, value)
}

impl View for TextFieldView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity(context);
        let initial_value = self.value.value();
        let placeholder = self.resolved_placeholder(|id| context.localized_message(id));
        let initial_placeholder = placeholder.clone();
        let secure = self.kind == TextFieldKind::Secure;
        let (retained, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let state = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(initial_placeholder)
                    .default_value(initial_value.clone())
                    .masked(secure)
            });
            RetainedTextField {
                state,
                owner_value: initial_value,
                last_selection: None,
                selection_before_preedit: None,
                secure,
                writer: Rc::new(RefCell::new(None)),
            }
        });
        let state = retained.read(context.app()).state.clone();
        let current_value = self.value.value();
        let show_clear_button =
            self.cleanable && !self.disabled && !self.readonly && !current_value.is_empty();
        let previous_owner_value = retained.read(context.app()).owner_value.clone();
        let previous_secure = retained.read(context.app()).secure;
        let retained_placeholder = state
            .read(context.app())
            .presentation()
            .placeholder()
            .to_string();
        if placeholder_changed(&retained_placeholder, &placeholder) {
            let placeholder = placeholder.clone();
            context.update_entity(&state, move |state, window, cx| {
                state.set_placeholder(placeholder, window, cx);
            });
        }
        let (retained_value, retained_masked, marked_range, current_selection) = context
            .update_entity(&state, |state, window, cx| {
                (
                    state.value().to_string(),
                    state.presentation().is_masked(),
                    state.marked_text_range(window, cx),
                    state.selected_range(),
                )
            });
        let last_selection = retained.read(context.app()).last_selection.clone();
        let selection_before_preedit = retained
            .read(context.app())
            .selection_before_preedit
            .clone();
        let owner_unchanged = current_value == previous_owner_value;
        let selection_to_restore = if marked_range.is_none()
            && owner_unchanged
            && retained_value != current_value
            && should_sync_owner_value(
                &current_value,
                &previous_owner_value,
                &retained_value,
                marked_range.clone(),
            ) {
            selection_before_preedit.clone()
        } else {
            None
        };
        // IME preedit updates InputState before InputEvent::Change writes the owner.
        // Keep it only while the owner still has its last synchronized value.
        if should_sync_owner_value(
            &current_value,
            &previous_owner_value,
            &retained_value,
            marked_range.clone(),
        ) {
            let value_to_sync = current_value.clone();
            let selection_to_restore = selection_to_restore.clone();
            context.update_entity(&state, move |state, window, cx| {
                state.set_value(value_to_sync, window, cx);
                if let Some(selection) = selection_to_restore {
                    state.set_selected_range(selection, cx);
                }
            });
        }
        let selection_after_sync =
            context.update_entity(&state, |state, _, _| state.selected_range());
        let next_selection_before_preedit = if marked_range.is_some() && owner_unchanged {
            selection_before_preedit
                .or_else(|| last_selection.or_else(|| Some(current_selection.clone())))
        } else {
            None
        };
        context.update_entity(&retained, move |retained, _, _| {
            retained.selection_before_preedit = next_selection_before_preedit;
            if retained.selection_before_preedit.is_none() {
                retained.last_selection = Some(selection_after_sync);
            }
        });
        if current_value != previous_owner_value || previous_secure != secure {
            context.update_entity(&retained, move |retained, _, _| {
                if current_value != retained.owner_value {
                    retained.owner_value = current_value;
                }
                retained.secure = secure;
            });
        }
        let next_masked = reconcile_masked_state(retained_masked, previous_secure, secure);
        if retained_masked != next_masked {
            context.update_entity(&state, move |state, window, cx| {
                state.set_masked(next_masked, window, cx);
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

        let mut input = Input::new(&state)
            .id(identity.to_string())
            .disabled(self.disabled)
            .readonly(self.readonly)
            // GPUI Components' built-in icon-only clear button has no accessible
            // label. Render the same affordance below with an explicit name.
            .cleanable(false)
            .tab_index(self.tab_index);
        input = match self.size {
            TextFieldSize::Small => input.small(),
            TextFieldSize::Large => input.large(),
        };
        if secure {
            input = input.content_type(InputContentType::Password);
        }
        input =
            input.aria_label(self.resolved_accessibility_label(|id| context.localized_message(id)));
        if secure || show_clear_button {
            let mut suffix = div().flex().items_center();
            if secure {
                let toggle_state = state.clone();
                let focus = state.clone();
                suffix = suffix.child(
                    GpuiButton::new(format!("{identity}-password-visibility"))
                        .icon(Icon::new(IconName::Eye))
                        .xsmall()
                        .text()
                        .tab_stop(true)
                        .disabled(self.disabled)
                        .toggled(visibility_control_is_toggled(next_masked))
                        .accessibility_label(self.resolved_password_visibility_accessibility_label(
                            |id| context.localized_message(id),
                        ))
                        .on_click(move |_, window, cx| {
                            toggle_state.update(cx, |state, cx| {
                                let masked = state.presentation().is_masked();
                                state.set_masked(!masked, window, cx);
                            });
                            focus.update(cx, |state, cx| state.focus(window, cx));
                        }),
                );
            }
            if show_clear_button {
                let clear = context.binding_writer(self.value.clone());
                let focus = state.clone();
                suffix = suffix.child(
                    GpuiButton::new(format!("{identity}-clear"))
                        .icon(Icon::new(IconName::Close))
                        .text()
                        .xsmall()
                        .tab_stop(CLEAR_BUTTON_TAB_STOP)
                        .accessibility_label(self.resolved_clear_button_accessibility_label(|id| {
                            context.localized_message(id)
                        }))
                        .on_click(move |_, window, cx| {
                            clear(String::new(), cx);
                            focus.update(cx, |state, cx| state.focus(window, cx));
                        }),
                );
            }
            input = input.suffix(suffix);
        }

        // Keep the native input and macOS rule on the same size selector.
        let mut selector = selector(context);
        selector.size = self.size.selector_value().into();
        let (component, part) = match self.kind {
            TextFieldKind::Plain | TextFieldKind::Secure => ("text_field", "body"),
            TextFieldKind::Search => ("search_field", "list_search"),
        };
        let style = required_style_with_artwork(context, component, part, &selector);
        if self.kind == TextFieldKind::Search {
            let mut icon = Icon::new(IconName::Search);
            if let Some(color) = style.colors.get("foreground") {
                icon = icon.text_color(crate::theme::registry::hsla(*color));
            }
            input = input.prefix(icon);
        }
        input = styled_element(input, &style);
        // Input draws its one-pixel border before applying the public Styled
        // refinement. The theme's border color can override that border here.
        if let Some(color) = style.colors.get("border") {
            input = input.border_color(crate::theme::registry::hsla(*color));
        }
        if self.disabled {
            let theme = crate::theme::registry::active_theme(context.app())
                .expect("a RustUI component theme requires an active theme");
            input = input.text_color(disabled_text_color(&theme.profile, theme.mode));
        }
        input.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Binding, CLEAR_BUTTON_TAB_STOP, TextField, TextFieldKind, TextFieldSize,
        clear_accessibility_label, disabled_text_color, placeholder_changed,
        reconcile_masked_state, replace_event_writer, should_sync_owner_value,
        visibility_control_is_toggled, write_with_current_writer,
    };
    use crate::{
        components::theme::selector_for_mode,
        theme::{AppearanceMode, parse_theme, resolve_component_style_with},
    };
    use rustui_core::Identity;
    use std::{cell::RefCell, rc::Rc};

    fn binding() -> Binding<String> {
        Binding::new(
            String::new(),
            |value: &String| value.clone(),
            |value, next| {
                *value = next;
            },
        )
    }

    #[test]
    #[should_panic(expected = "TextField IDs must be nonempty")]
    fn rejects_empty_explicit_identity() {
        TextField("Name", binding()).id("");
    }

    #[test]
    #[should_panic(expected = "TextField IDs must be nonempty")]
    fn rejects_whitespace_only_explicit_identity() {
        TextField("Name", binding()).id(" \t ");
    }

    #[test]
    fn preserves_nonblank_explicit_identity_verbatim() {
        let identity = Identity::from(" field-name ");
        assert_eq!(
            TextField("Name", binding()).id(identity.clone()).id,
            Some(identity)
        );
    }

    #[test]
    fn implicit_identity_survives_placeholder_localization_and_separates_siblings() {
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };
        let original = TextField("Name", binding()).identity_with(allocate_at_slot(0));
        let translated = TextField("Nom", binding()).identity_with(allocate_at_slot(0));
        let sibling = TextField("Name", binding()).identity_with(allocate_at_slot(1));

        assert_eq!(original, translated);
        assert_ne!(original, sibling);
    }

    #[test]
    fn search_field_identity_survives_placeholder_localization_and_separates_siblings() {
        use crate::components::input::search_field::SearchField;

        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };
        let original = SearchField("Search files", binding()).identity_with(allocate_at_slot(0));
        let translated = SearchField("Rechercher", binding()).identity_with(allocate_at_slot(0));
        let sibling = SearchField("Search files", binding()).identity_with(allocate_at_slot(1));

        assert_eq!(original, translated);
        assert_ne!(original, sibling);
    }

    #[test]
    fn retained_input_event_uses_the_latest_owner_binding_writer() {
        let slot = RefCell::new(None);
        let old_called = Rc::new(RefCell::new(false));
        let new_called = Rc::new(RefCell::new(false));
        let old = old_called.clone();
        replace_event_writer(
            &slot,
            Rc::new(move |_: String, _: &mut ()| *old.borrow_mut() = true),
        );
        let new = new_called.clone();
        replace_event_writer(
            &slot,
            Rc::new(move |_: String, _: &mut ()| *new.borrow_mut() = true),
        );

        write_with_current_writer(&slot, "typed".to_owned(), &mut ());

        assert!(!*old_called.borrow());
        assert!(*new_called.borrow());
    }

    #[test]
    fn native_input_size_selects_matching_macos_rule() {
        let document = parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            for (size, expected_width) in
                [(TextFieldSize::Small, 223.0), (TextFieldSize::Large, 229.0)]
            {
                let mut selector = selector_for_mode(mode);
                selector.size = size.selector_value().into();
                let style = resolve_component_style_with(
                    &document,
                    "text_field",
                    "body",
                    &selector,
                    &["filter", "blend_mode"],
                )
                .unwrap();
                assert_eq!(style.lengths["width"], expected_width);
            }
        }
    }

    #[test]
    fn clear_button_accessible_name_uses_the_field_placeholder() {
        assert_eq!(clear_accessibility_label("Search"), "Clear Search");
    }

    #[test]
    fn clear_button_accessible_name_stays_meaningful_without_a_placeholder() {
        assert_eq!(clear_accessibility_label(""), "Clear text field");
        assert_eq!(clear_accessibility_label(" \t "), "Clear text field");
    }

    #[test]
    fn clear_button_participates_in_keyboard_tab_traversal() {
        assert!(CLEAR_BUTTON_TAB_STOP);
    }

    #[test]
    fn localized_clear_button_accessible_name_overrides_english_fallback() {
        let field =
            TextField("Search", binding()).clear_button_accessibility_label("Effacer la recherche");

        assert_eq!(
            field.resolved_clear_button_accessibility_label(|_| unreachable!()),
            "Effacer la recherche"
        );
    }

    #[test]
    fn localized_clear_button_accessible_name_uses_catalog_and_keeps_fallback() {
        let field = TextField("Search", binding())
            .localized_clear_button_accessibility_label("rustui-text-field-clear-button");

        assert_eq!(
            field.resolved_clear_button_accessibility_label(|id| {
                assert_eq!(id, "rustui-text-field-clear-button");
                "Effacer le texte".to_owned()
            }),
            "Effacer le texte"
        );
        assert_eq!(
            field.resolved_clear_button_accessibility_label(|id| id.to_owned()),
            "Clear Search"
        );
        assert_eq!(
            field.resolved_clear_button_accessibility_label(|_| " \t".to_owned()),
            "Clear Search"
        );
    }

    #[test]
    fn localized_clear_button_accessible_name_preserves_explicit_fallback() {
        let field = TextField("Search", binding())
            .clear_button_accessibility_label("Clear query")
            .localized_clear_button_accessibility_label("rustui-text-field-clear-button");

        assert_eq!(
            field.resolved_clear_button_accessibility_label(|id| id.to_owned()),
            "Clear query"
        );
    }

    #[test]
    #[should_panic(
        expected = "TextField clear button accessibility localization key must not be blank"
    )]
    fn localized_clear_button_accessible_name_rejects_blank_message_id() {
        TextField("Search", binding()).localized_clear_button_accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "TextField clear button accessibility label must not be blank")]
    fn clear_button_accessible_name_must_not_be_blank() {
        TextField("Search", binding()).clear_button_accessibility_label(" \t ");
    }

    #[test]
    fn secure_field_visibility_control_has_a_stable_accessible_name() {
        let field = TextField("Password", binding()).with_kind(TextFieldKind::Secure);

        assert_eq!(
            field.resolved_password_visibility_accessibility_label(|_| unreachable!()),
            "Toggle password visibility"
        );
    }

    #[test]
    fn secure_field_visibility_control_accepts_a_localized_name() {
        let field = TextField("Password", binding())
            .with_kind(TextFieldKind::Secure)
            .password_visibility_accessibility_label("Afficher ou masquer le mot de passe");

        assert_eq!(
            field.resolved_password_visibility_accessibility_label(|_| unreachable!()),
            "Afficher ou masquer le mot de passe"
        );
    }

    #[test]
    fn localized_password_visibility_name_resolves_catalog_and_preserves_fallback() {
        let default = TextField("Password", binding())
            .localized_password_visibility_accessibility_label("secure_field.toggle_visibility");
        assert_eq!(
            default.resolved_password_visibility_accessibility_label(|id| {
                assert_eq!(id, "secure_field.toggle_visibility");
                "Afficher ou masquer le mot de passe".into()
            }),
            "Afficher ou masquer le mot de passe"
        );
        assert_eq!(
            default.resolved_password_visibility_accessibility_label(|id| id.to_owned()),
            "Toggle password visibility"
        );
        assert_eq!(
            default.resolved_password_visibility_accessibility_label(|_| " \t".to_owned()),
            "Toggle password visibility"
        );

        let explicit = TextField("Password", binding())
            .password_visibility_accessibility_label("Show or hide password")
            .localized_password_visibility_accessibility_label("secure_field.toggle_visibility");
        assert_eq!(
            explicit.resolved_password_visibility_accessibility_label(|_| String::new()),
            "Show or hide password"
        );
    }

    #[test]
    #[should_panic(
        expected = "TextField password visibility accessibility localization key must not be blank"
    )]
    fn localized_password_visibility_accessible_name_key_must_not_be_blank() {
        TextField("Password", binding()).localized_password_visibility_accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(
        expected = "TextField password visibility accessibility label must not be blank"
    )]
    fn secure_field_visibility_accessible_name_must_not_be_blank() {
        TextField("Password", binding()).password_visibility_accessibility_label(" \t ");
    }

    #[test]
    fn secure_field_visibility_toggle_survives_same_mode_rerenders() {
        assert!(!reconcile_masked_state(false, true, true));
    }

    #[test]
    fn secure_field_mode_transitions_reset_masking_to_the_new_mode_default() {
        assert!(!reconcile_masked_state(true, true, false));
        assert!(reconcile_masked_state(false, false, true));
    }

    #[test]
    fn secure_field_visibility_control_exposes_revealed_state() {
        assert!(!visibility_control_is_toggled(true));
        assert!(visibility_control_is_toggled(false));
    }

    #[test]
    fn explicit_accessible_name_preserves_nonblank_text() {
        let field = TextField("Search", binding()).accessibility_label("  Find items  ");

        assert_eq!(
            field
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            "  Find items  "
        );
    }

    #[test]
    fn localized_accessible_name_resolves_and_preserves_literal_fallback() {
        let field = TextField("Search", binding())
            .accessibility_label("Find items")
            .localized_accessibility_label("field.search");
        assert_eq!(
            field
                .resolved_accessibility_label(|_| "Rechercher".into())
                .as_ref(),
            "Rechercher"
        );
        assert_eq!(
            field.resolved_accessibility_label(|id| id.into()).as_ref(),
            "Find items"
        );
    }

    #[test]
    fn localized_accessible_name_falls_back_to_placeholder_then_default() {
        let field = TextField("Search", binding()).localized_accessibility_label("field.search");
        assert_eq!(
            field.resolved_accessibility_label(|id| id.into()).as_ref(),
            "Search"
        );
        let blank = TextField(" \t", binding()).localized_accessibility_label("field.search");
        assert_eq!(
            blank.resolved_accessibility_label(|id| id.into()).as_ref(),
            "Text field"
        );
    }

    #[test]
    fn localized_placeholder_updates_visible_text_and_default_accessible_name() {
        let field = TextField("Search", binding()).localized_placeholder("field.search");
        assert_eq!(
            field.resolved_placeholder(|_| "Rechercher".into()).as_ref(),
            "Rechercher"
        );
        assert_eq!(
            field
                .resolved_accessibility_label(|_| "Rechercher".into())
                .as_ref(),
            "Rechercher"
        );
        assert_eq!(
            field.resolved_placeholder(|id| id.into()).as_ref(),
            "Search"
        );

        let explicitly_named = TextField("Search", binding())
            .accessibility_label("Find items")
            .localized_placeholder("field.search");
        assert_eq!(
            explicitly_named
                .resolved_accessibility_label(|_| "Rechercher".into())
                .as_ref(),
            "Find items"
        );

        let replaced = TextField("Search", binding())
            .localized_placeholder("field.old")
            .localized_placeholder("field.new");
        assert_eq!(
            replaced
                .resolved_accessibility_label(|id| {
                    assert_eq!(id, "field.new");
                    "Chercher".into()
                })
                .as_ref(),
            "Chercher"
        );
    }

    #[test]
    fn padded_localization_key_echo_uses_text_field_fallback() {
        let field = TextField("Search", binding()).localized_accessibility_label("field.search");

        assert_eq!(
            field
                .resolved_accessibility_label(|_| " \tfield.search\n".to_owned())
                .as_ref(),
            "Search"
        );
    }

    #[test]
    #[should_panic(expected = "TextField accessibility localization key must not be blank")]
    fn localized_accessible_name_rejects_blank_message_id() {
        TextField("Search", binding()).localized_accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "TextField accessibility label must not be empty")]
    fn explicit_accessible_name_must_not_be_blank() {
        TextField("Search", binding()).accessibility_label("\t ");
    }

    #[test]
    fn disabled_text_uses_the_platform_token_in_both_appearances() {
        let profile = parse_theme(include_str!("../../../../../themes/macos27.json"))
            .unwrap()
            .platform_snapshot;
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            assert_eq!(
                disabled_text_color(&profile, mode),
                gpui_component::theme::try_parse_color(&profile.colors(Some(mode)).disabled_text)
                    .unwrap()
            );
        }
    }

    #[test]
    fn marked_preedit_replacing_selection_does_not_sync_unchanged_owner() {
        // The preedit replaced selected "bc"; the original owner value is "abcd".
        assert!(!should_sync_owner_value("abcd", "abcd", "aにd", Some(1..2)));
    }

    #[test]
    fn repeated_marked_preedit_does_not_sync_unchanged_owner() {
        assert!(!should_sync_owner_value(
            "abcd",
            "abcd",
            "a日本d",
            Some(1..3)
        ));
    }

    #[test]
    fn canceled_preedit_without_a_marked_range_restores_unchanged_owner() {
        // A canceled composition can leave the retained state different from
        // its owner after GPUI clears the marked range. The owner remains the
        // committed value, so reconciliation must restore it.
        assert!(should_sync_owner_value("abcd", "abcd", "aにd", None));
    }

    #[test]
    fn unchanged_value_does_not_call_set_value_and_reset_selection() {
        // A selection is editor state, not part of the string. With no owner
        // change, reconciliation must leave InputState untouched.
        assert!(!should_sync_owner_value("abcdef", "abcdef", "abcdef", None));
    }

    #[test]
    fn changed_placeholder_is_refreshed_in_retained_input_state() {
        assert!(placeholder_changed("Search", "Find"));
        assert!(!placeholder_changed("Search", "Search"));
    }

    #[test]
    fn committed_input_matching_owner_does_not_sync_again() {
        assert!(!should_sync_owner_value("a日本d", "abcd", "a日本d", None));
    }

    #[test]
    fn external_owner_update_during_preedit_is_synchronized() {
        assert!(should_sync_owner_value(
            "external update",
            "abcd",
            "aにd",
            Some(1..2)
        ));
    }

    #[test]
    fn external_owner_update_without_preedit_is_synchronized() {
        assert!(should_sync_owner_value(
            "external update",
            "abcd",
            "abcd",
            None
        ));
    }
}
