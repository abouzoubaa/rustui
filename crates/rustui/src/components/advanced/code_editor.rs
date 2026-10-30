//! Controlled source editing backed by the official retained EditorState.

use std::{cell::RefCell, rc::Rc};

use gpui::{AnyElement, AppContext as _, Entity, IntoElement, SharedString, px};
use gpui_component::input::{Editor, EditorState, InputEvent};
use rustui_core::{Binding, Identity};

use crate::view::{EmptyView, RenderContext, View};

fn resolved_message_label(id: &str, localized: String, fallback: &SharedString) -> SharedString {
    if !localized.trim().is_empty() && localized.trim() != id.trim() {
        SharedString::from(localized)
    } else {
        fallback.clone()
    }
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

struct RetainedCodeEditor {
    editor: Entity<EditorState>,
    line_numbers: bool,
    writer: Rc<RefCell<Option<EventWriter<String, gpui::App>>>>,
}

/// A controlled native code editor with stable caller-supplied identity.
///
/// Language selection controls upstream editing behavior. Native builds include
/// Rust and JSON syntax grammars; Web grammars are not yet enabled. This
/// adapter does not configure a language server.
pub struct CodeEditorView {
    id: Identity,
    value: Binding<String>,
    language: SharedString,
    placeholder: SharedString,
    placeholder_message_id: Option<SharedString>,
    line_numbers: bool,
    height: f32,
    disabled: bool,
    readonly: bool,
    accessibility_label: Option<SharedString>,
    accessibility_label_message_id: Option<SharedString>,
    tab_index: isize,
}

/// Creates an editor whose value belongs to its enclosing component.
///
/// Use a stable unique ID for each editor and keep that ID associated with the
/// same binding field. Omitting the editor releases its retained native state.
#[allow(non_snake_case)]
pub fn CodeEditor(id: impl Into<Identity>, value: Binding<String>) -> CodeEditorView {
    CodeEditorView::new(id, value)
}

impl CodeEditorView {
    /// Creates a 240-point editor with line numbers and plain-text editing.
    /// Panics if `id` contains only whitespace or is empty.
    pub fn new(id: impl Into<Identity>, value: Binding<String>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "CodeEditor ID must be nonempty"
        );
        Self {
            id,
            value,
            language: "text".into(),
            placeholder: SharedString::default(),
            placeholder_message_id: None,
            line_numbers: true,
            height: 240.0,
            disabled: false,
            readonly: false,
            accessibility_label: None,
            accessibility_label_message_id: None,
            tab_index: 0,
        }
    }

    /// Selects upstream language editing rules without replacing the editor.
    /// Unknown language names use the upstream fallback rules. Panics for an
    /// empty or whitespace-only name. Other grammar availability depends on features.
    pub fn language(mut self, language: impl Into<SharedString>) -> Self {
        let language = language.into();
        assert!(
            !language.trim().is_empty(),
            "CodeEditor language must be nonempty"
        );
        self.language = language;
        self
    }

    /// Shows or hides the upstream line-number gutter without resetting text.
    pub fn line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
        self
    }

    /// Sets a finite, positive editor height in logical points.
    pub fn height(mut self, height: f32) -> Self {
        assert!(
            height.is_finite() && height > 0.0,
            "CodeEditor height must be finite and positive"
        );
        self.height = height;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self.placeholder_message_id = None;
        self
    }

    /// Resolves the placeholder through the active locale catalog.
    pub fn localized_placeholder(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "CodeEditor placeholder localization key must not be blank"
        );
        self.placeholder_message_id = Some(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Prevents user edits while preserving native focus, selection, and copy.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "CodeEditor accessibility label must not be empty"
        );
        self.accessibility_label = Some(label);
        self.accessibility_label_message_id = None;
        self
    }

    /// Resolves the editor's accessible name through the active locale catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "CodeEditor accessibility localization key must not be blank"
        );
        self.accessibility_label_message_id = Some(id);
        self
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }
}

impl View for CodeEditorView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let current_value = self.value.value();
        let resolved_placeholder = match &self.placeholder_message_id {
            Some(id) => resolved_message_label(
                id.as_ref(),
                context.localized_message(id.as_ref()),
                &self.placeholder,
            ),
            None => self.placeholder.clone(),
        };
        let resolved_accessibility_label = match &self.accessibility_label_message_id {
            Some(id) => resolved_message_label(
                id.as_ref(),
                context.localized_message(id.as_ref()),
                self.accessibility_label
                    .as_ref()
                    .unwrap_or(&self.placeholder),
            ),
            None => self.accessibility_label.clone().unwrap_or_default(),
        };
        let (retained, created) =
            context.retained_entity(self.id.clone(), |window, cx| RetainedCodeEditor {
                editor: cx.new(|cx| {
                    EditorState::new(window, cx)
                        .language(self.language.clone())
                        .placeholder(resolved_placeholder.clone())
                        .line_number(self.line_numbers)
                        .default_value(current_value.clone())
                }),
                line_numbers: self.line_numbers,
                writer: Rc::new(RefCell::new(None)),
            });
        let retained_state = retained.read(context.app());
        let editor = retained_state.editor.clone();
        let line_numbers_changed = retained_state.line_numbers != self.line_numbers;
        let native = editor.read(context.app());
        let value_changed = native.value().to_string() != current_value;
        let language_changed = native.language_name() != self.language;
        let placeholder_changed = native.presentation().placeholder() != &resolved_placeholder;
        if value_changed || language_changed || placeholder_changed || line_numbers_changed {
            context.update_entity(&editor, |native, window, cx| {
                if value_changed {
                    native.set_value(current_value, window, cx);
                }
                if language_changed {
                    native.set_highlighter(self.language.clone(), cx);
                }
                if placeholder_changed {
                    native.set_placeholder(resolved_placeholder.clone(), window, cx);
                }
                if line_numbers_changed {
                    native.set_line_number(self.line_numbers, window, cx);
                }
            });
            if line_numbers_changed {
                context.update_entity(&retained, |retained, _, _| {
                    retained.line_numbers = self.line_numbers
                });
            }
        }
        if created {
            let writer = retained.read(context.app()).writer.clone();
            context.subscribe_retained(&self.id, &editor, move |editor, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    write_with_current_writer(&writer, editor.read(cx).value().to_string(), cx);
                }
            });
        }
        let writer = retained.read(context.app()).writer.clone();
        if self.disabled || self.readonly {
            *writer.borrow_mut() = None;
        } else {
            replace_event_writer(&writer, context.binding_writer(self.value.clone()));
        }
        let mut editor_view = Editor::new(&editor)
            .h(px(self.height))
            .disabled(self.disabled)
            .readonly(self.readonly)
            .tab_index(self.tab_index);
        if !resolved_accessibility_label.is_empty() {
            editor_view = editor_view.aria_label(resolved_accessibility_label);
        }
        editor_view.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn retained_editor_ignores_late_changes_when_disabled_or_readonly(
        cx: &mut gpui::TestAppContext,
    ) {
        use crate::runtime::component_host::ComponentHost;
        struct Owner {
            value: String,
            disabled: bool,
            readonly: bool,
        }
        impl View for Owner {
            fn body(&self) -> impl View {
                CodeEditor(
                    "editor-guard",
                    Binding::new(
                        self.value.clone(),
                        |owner: &Owner| owner.value.clone(),
                        |owner, value| owner.value = value,
                    ),
                )
                .disabled(self.disabled)
                .readonly(self.readonly)
            }
        }
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (owner, window) = cx.add_window_view(|_, cx| {
            ComponentHost::new(
                Owner {
                    value: "initial".into(),
                    disabled: false,
                    readonly: false,
                },
                cx,
            )
        });
        let editor = window.update(|window, app| {
            let _ = window.draw(app);
            owner
                .read(app)
                .retained_entity_for_test::<RetainedCodeEditor>(
                    vec![],
                    Identity::from("editor-guard"),
                )
                .unwrap()
                .read(app)
                .editor
                .clone()
        });
        window.update(|window, app| {
            owner.update(app, |host, cx| {
                host.component_mut().disabled = true;
                host.component_mut().value = "external".into();
                cx.notify();
            });
            let _ = window.draw(app);
            assert_eq!(editor.read(app).value().to_string(), "external");
            editor.update(app, |editor, cx| {
                editor.set_value("late", window, cx);
                cx.emit(InputEvent::Change);
            });
        });
        window.cx.run_until_parked();
        window.update(|_, app| assert_eq!(owner.read(app).component().value, "external"));
        window.update(|window, app| {
            owner.update(app, |host, cx| {
                host.component_mut().disabled = false;
                cx.notify();
            });
            let _ = window.draw(app);
            editor.update(app, |editor, cx| {
                editor.set_value("fresh", window, cx);
                cx.emit(InputEvent::Change);
            });
        });
        window.cx.run_until_parked();
        window.update(|_, app| assert_eq!(owner.read(app).component().value, "fresh"));
        window.update(|window, app| {
            owner.update(app, |host, cx| {
                host.component_mut().readonly = true;
                cx.notify();
            });
            let _ = window.draw(app);
            editor.update(app, |editor, cx| {
                editor.set_value("late readonly", window, cx);
                cx.emit(InputEvent::Change);
            });
        });
        window.cx.run_until_parked();
        window.update(|_, app| assert_eq!(owner.read(app).component().value, "fresh"));
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn native_rust_and_json_highlighters_parse_source() {
        use gpui_base::input::{HighlightStyleResolver, Rope};
        use gpui_component::highlighter::SyntaxHighlighter;

        struct AllStyles;
        impl HighlightStyleResolver for AllStyles {
            fn style(&self, _name: &str) -> Option<gpui::HighlightStyle> {
                Some(gpui::HighlightStyle::default())
            }
        }
        for (language, source) in [
            ("rust", "fn main() { let answer = 42; }"),
            ("json", r#"{"answer": 42}"#),
        ] {
            let mut highlighter = SyntaxHighlighter::new(language);
            assert_eq!(highlighter.language().as_ref(), language);
            assert!(highlighter.update(None, &Rope::from_str(source), None));
            let tree = highlighter
                .tree()
                .expect("grammar must produce a syntax tree");
            assert!(
                !tree.root_node().has_error(),
                "{language} source should parse"
            );
            assert!(
                !highlighter
                    .styles(&(0..source.len()), &AllStyles)
                    .is_empty(),
                "{language} highlight query should match source"
            );
        }
    }

    #[test]
    fn retained_editor_event_uses_latest_owner_binding_writer() {
        let slot = RefCell::new(None);
        let old_writer_called = Rc::new(std::cell::Cell::new(false));
        let new_writer_called = Rc::new(std::cell::Cell::new(false));

        let old = old_writer_called.clone();
        replace_event_writer(&slot, Rc::new(move |_: String, _: &mut ()| old.set(true)));
        let new = new_writer_called.clone();
        replace_event_writer(&slot, Rc::new(move |_: String, _: &mut ()| new.set(true)));

        write_with_current_writer(&slot, "edited".to_owned(), &mut ());

        assert!(!old_writer_called.get());
        assert!(new_writer_called.get());
    }

    struct Owner {
        text: String,
    }
    fn binding() -> Binding<String> {
        Binding::new::<Owner>(
            "initial".into(),
            |owner| owner.text.clone(),
            |owner, value| owner.text = value,
        )
    }

    #[test]
    fn defaults_and_configuration_keep_identity_and_binding() {
        let editor = CodeEditor("source", binding());
        assert_eq!(editor.accessibility_label, None);
        assert_eq!(editor.language.as_ref(), "text");
        assert!(editor.line_numbers);
        assert_eq!(editor.height, 240.0);
        assert!(!editor.disabled && !editor.readonly);
        let editor = editor
            .language("rust")
            .line_numbers(false)
            .height(320.0)
            .placeholder("Source")
            .disabled(true)
            .readonly(true)
            .accessibility_label("Document source")
            .tab_index(-1);
        assert_eq!(editor.id, Identity::from("source"));
        assert_eq!(editor.value.value(), "initial");
        assert_eq!(editor.language.as_ref(), "rust");
        assert!(!editor.line_numbers);
        assert_eq!(editor.height, 320.0);
        assert_eq!(editor.placeholder.as_ref(), "Source");
        assert!(editor.disabled && editor.readonly);
        assert_eq!(
            editor.accessibility_label.as_deref(),
            Some("Document source")
        );
        assert_eq!(editor.tab_index, -1);
    }

    #[test]
    fn accessible_name_preserves_nonblank_text_verbatim() {
        let editor = CodeEditor("source", binding()).accessibility_label("  Document source  ");

        assert_eq!(
            editor.accessibility_label.as_deref(),
            Some("  Document source  ")
        );
    }

    #[test]
    #[should_panic(expected = "CodeEditor accessibility label must not be empty")]
    fn rejects_blank_explicit_accessible_name() {
        CodeEditor("source", binding()).accessibility_label(" \t\n ");
    }

    #[test]
    fn rejects_invalid_height_and_empty_identity_or_language() {
        for height in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                std::panic::catch_unwind(|| CodeEditor("source", binding()).height(height))
                    .is_err()
            );
        }
        for empty in ["", "  "] {
            assert!(std::panic::catch_unwind(|| CodeEditor(empty, binding())).is_err());
            assert!(
                std::panic::catch_unwind(|| CodeEditor("source", binding()).language(empty))
                    .is_err()
            );
        }
    }
}
