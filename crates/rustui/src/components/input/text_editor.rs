//! Multiline text editor backed by a retained GPUI Components TextareaState.

use gpui::{AnyElement, IntoElement, SharedString, px};
use gpui_component::input::{InputEvent, Textarea, TextareaState};
use rustui_core::{Binding, Identity};

use crate::view::{EmptyView, RenderContext, View};

/// A controlled multiline text editor whose editing state survives body rebuilds.
pub struct TextEditorView {
    placeholder: SharedString,
    value: Binding<String>,
    id: Option<Identity>,
    height: f32,
    disabled: bool,
    readonly: bool,
    accessibility_label: Option<SharedString>,
    tab_index: isize,
}

impl TextEditorView {
    pub fn new(placeholder: impl Into<SharedString>, value: Binding<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
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
        self.id = Some(id.into());
        self
    }

    pub fn height(mut self, height: f32) -> Self {
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

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    fn identity(&self) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| Identity::from(format!("text-editor:{}", self.placeholder)))
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
        let identity = self.identity();
        let initial_value = self.value.value();
        let placeholder = self.placeholder.clone();
        let (state, created) = context.retained_entity(identity.clone(), move |window, cx| {
            TextareaState::new(window, cx)
                .placeholder(placeholder)
                .default_value(initial_value)
        });

        let current_value = self.value.value();
        let retained_value = state.read(context.app()).value().to_string();
        if retained_value != current_value {
            context.update_entity(&state, move |state, window, cx| {
                state.set_value(current_value, window, cx);
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

        let mut editor = Textarea::new(&state)
            .h(px(self.height))
            .disabled(self.disabled)
            .readonly(self.readonly)
            .tab_index(self.tab_index);
        if let Some(label) = &self.accessibility_label {
            editor = editor.aria_label(label.clone());
        }
        editor.into_any_element()
    }
}
