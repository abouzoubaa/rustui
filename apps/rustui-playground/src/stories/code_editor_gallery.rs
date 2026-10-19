//! Retained native source editing through the public RustUI API.

use rustui::{
    component,
    prelude::{AnyView, Button, CodeEditor, Component, HStack, Text, VStack, View},
};

const RUST_SOURCE: &str = "fn main() {\n    println!(\"Hello, RustUI\");\n}\n";
const JSON_SOURCE: &str = "{\n  \"message\": \"Hello, RustUI\"\n}\n";

pub fn demo() -> impl View {
    Component("code-editor-gallery", CodeEditorGallery::default)
}

#[component]
struct CodeEditorGallery {
    #[state(String::from(RUST_SOURCE))]
    source: String,
    #[state(false)]
    json_language: bool,
    #[state(true)]
    line_numbers: bool,
    #[state(false)]
    readonly: bool,
    #[state(false)]
    disabled: bool,
    #[state(true)]
    mounted: bool,
    #[state(0)]
    external_replacements: u32,
}

impl CodeEditorGallery {
    fn toggle_language(&mut self) {
        self.json_language = !self.json_language;
    }
    fn toggle_line_numbers(&mut self) {
        self.line_numbers = !self.line_numbers;
    }
    fn toggle_readonly(&mut self) {
        self.readonly = !self.readonly;
    }
    fn toggle_disabled(&mut self) {
        self.disabled = !self.disabled;
    }
    fn toggle_mount(&mut self) {
        self.mounted = !self.mounted;
    }
    fn reset(&mut self) {
        self.source = if self.json_language {
            JSON_SOURCE
        } else {
            RUST_SOURCE
        }
        .into();
    }
    fn replace_external(&mut self) {
        self.external_replacements += 1;
        self.source = if self.json_language {
            format!(
                "{{\n  \"external_replacement\": {}\n}}\n",
                self.external_replacements
            )
        } else {
            format!(
                "fn replacement_{}() {{\n    println!(\"Externally updated\");\n}}\n",
                self.external_replacements
            )
        };
    }
}

impl View for CodeEditorGallery {
    fn body(&self) -> impl View {
        let language = if self.json_language() { "json" } else { "rust" };
        let editor = if self.mounted() {
            AnyView::new(
                CodeEditor("gallery-source", self.source_binding())
                    .language(language)
                    .line_numbers(self.line_numbers())
                    .height(180.0)
                    .placeholder("Enter source code")
                    .readonly(self.readonly())
                    .disabled(self.disabled())
                    .accessibility_label("Source code")
                    .tab_index(0),
            )
        } else {
            AnyView::new(Text(
                "Source editor omitted; the bound value remains in its owner.",
            ))
        };
        let source = self.source();
        VStack::new().spacing(8.0)
            .child(Text("Source code"))
            .child(Text("Syntax grammars are disabled; syntax highlighting awaits dependency approval."))
            .child(Text(format!("Language: {} · lines: {} · readonly: {} · disabled: {} · mounted: {}", language, self.line_numbers(), self.readonly(), self.disabled(), self.mounted())))
            .child(HStack::new().spacing(6.0)
                .child(Button("Rust / JSON setting", self.action(Self::toggle_language)).id("code-editor-language"))
                .child(Button("Toggle line numbers", self.action(Self::toggle_line_numbers)).id("code-editor-lines"))
                .child(Button("Toggle readonly", self.action(Self::toggle_readonly)).id("code-editor-readonly")))
            .child(HStack::new().spacing(6.0)
                .child(Button("Toggle disabled", self.action(Self::toggle_disabled)).id("code-editor-disabled"))
                .child(Button("Reset source", self.action(Self::reset)).id("code-editor-reset"))
                .child(Button("Replace externally", self.action(Self::replace_external)).id("code-editor-replace"))
                .child(Button("Omit / remount", self.action(Self::toggle_mount)).id("code-editor-mount")))
            .child(editor)
            .child(Text(format!("Bound source ({} characters):\n{}", source.chars().count(), source)))
            .child(Text("Changing language keeps the buffer. Reset uses the selected language's sample. Omission retains the owner value; remount creates fresh native editing state."))
    }
}
