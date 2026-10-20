//! Single-line text input backed by a retained GPUI Components InputState.

use gpui::{AnyElement, AppContext, Entity, EntityInputHandler, IntoElement, SharedString, Styled};
use gpui_component::{
    Icon, IconName, Sizable,
    input::{Input, InputContentType, InputEvent, InputState},
};
use rustui_core::{Binding, Identity};
use std::ops::Range;

use crate::{
    components::theme::{required_style_with_artwork, selector, styled_element},
    view::{EmptyView, RenderContext, View},
};

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
    value: Binding<String>,
    id: Option<Identity>,
    kind: TextFieldKind,
    size: TextFieldSize,
    disabled: bool,
    readonly: bool,
    cleanable: bool,
    accessibility_label: Option<SharedString>,
    tab_index: isize,
}

impl TextFieldView {
    pub fn new(placeholder: impl Into<SharedString>, value: Binding<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            value,
            id: None,
            kind: TextFieldKind::Plain,
            size: TextFieldSize::Small,
            disabled: false,
            readonly: false,
            cleanable: false,
            accessibility_label: None,
            tab_index: 0,
        }
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
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
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
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            let kind = match self.kind {
                TextFieldKind::Plain => "text-field",
                TextFieldKind::Secure => "secure-field",
                TextFieldKind::Search => "search-field",
            };
            Identity::from(format!("{kind}:{}", self.placeholder))
        })
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
        let identity = self.identity();
        let initial_value = self.value.value();
        let placeholder = self.placeholder.clone();
        let secure = self.kind == TextFieldKind::Secure;
        let (retained, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let state = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder)
                    .default_value(initial_value.clone())
                    .masked(secure)
            });
            RetainedTextField {
                state,
                owner_value: initial_value,
            }
        });
        let state = retained.read(context.app()).state.clone();
        let current_value = self.value.value();
        let previous_owner_value = retained.read(context.app()).owner_value.clone();
        let (retained_value, retained_masked, marked_range) =
            context.update_entity(&state, |state, window, cx| {
                (
                    state.value().to_string(),
                    state.presentation().is_masked(),
                    state.marked_text_range(window, cx),
                )
            });
        // IME preedit updates InputState before InputEvent::Change writes the owner.
        // Keep it only while the owner still has its last synchronized value.
        if should_sync_owner_value(
            &current_value,
            &previous_owner_value,
            &retained_value,
            marked_range,
        ) {
            let value_to_sync = current_value.clone();
            context.update_entity(&state, move |state, window, cx| {
                state.set_value(value_to_sync, window, cx);
            });
        }
        if current_value != previous_owner_value {
            context.update_entity(&retained, move |retained, _, _| {
                retained.owner_value = current_value;
            });
        }
        if retained_masked != secure {
            context.update_entity(&state, move |state, window, cx| {
                state.set_masked(secure, window, cx);
            });
        }

        if created {
            let write = context.binding_writer(self.value.clone());
            context.subscribe_retained(&identity, &state, move |state, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    write(state.read(cx).value().to_string(), cx);
                }
            });
        }

        let mut input = Input::new(&state)
            .id(identity.to_string())
            .disabled(self.disabled)
            .readonly(self.readonly)
            .cleanable(self.cleanable)
            .tab_index(self.tab_index);
        input = match self.size {
            TextFieldSize::Small => input.small(),
            TextFieldSize::Large => input.large(),
        };
        if secure {
            input = input.content_type(InputContentType::Password).mask_toggle();
        }
        if let Some(label) = &self.accessibility_label {
            input = input.aria_label(label.clone());
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
        input.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{TextFieldSize, should_sync_owner_value};
    use crate::{
        components::theme::selector_for_mode,
        theme::{AppearanceMode, parse_theme, resolve_component_style_with},
    };

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
