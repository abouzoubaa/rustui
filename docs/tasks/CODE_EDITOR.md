# Controlled native CodeEditor adapter

## Current syntax support — 2026-10-28

Native RustUI builds enable the pinned Components Rust and JSON grammars. A
focused test parses representative Rust and JSON and verifies their highlight
queries return styles; this is source evidence, not a native visual acceptance
result. The Web build keeps syntax highlighting disabled and reports that
limitation in Playground. No language server is configured. The earlier proposal
and no-highlighting observations below describe the previous configuration.

`CodeEditor(id, Binding<String>)` adapts the pinned GPUI Components `Editor`
and `EditorState`. It uses the enclosing component's retained registry and
binding writer, matching the existing TextEditor ownership path.

```rust
CodeEditor("source", self.source_binding())
    .language("rust")
    .line_numbers(true)
    .height(240.0)
    .accessibility_label("Document source")
```

The ID is explicit, stable, nonempty, and unique in its retention scope. Keep
it associated with the same owner binding field. The default language is
`text`, height is 240 logical points, and line numbers are enabled. Language
names must be nonempty; unknown names use upstream fallback editing rules.
Height must be finite and positive. Builders also support placeholder,
readonly, disabled, accessibility label, and tab index.

## Retention and updates

A private retained record owns the native `Entity<EditorState>` and the last
line-number configuration. Owner body rebuilds reuse that editor, preserving
its native editing state. Value changes from the binding synchronize only when
different. Native `InputEvent::Change` writes back through the existing owner
binding writer; one retained subscription is installed on creation.

Language changes call native `set_highlighter` after comparing `language_name`.
Placeholder changes use native getter/setter. Line-number changes use
`set_line_number` only when the tracked flag differs; repeated identical
configuration does not continuously notify the editor. These updates reuse
the same native entity. Readonly and disabled behavior, selection, caret,
history, scrolling, theme styling, and monospace font choice stay upstream.
The adapter assumes no additional font or grammar asset bundle. Removing the
control releases its registry entry and subscription; it stores no owner
entity reference or separate editing runtime.

Relevant source paths:

- `crates/rustui/src/components/advanced/code_editor.rs`: adapter and configuration checks.
- `crates/rustui/src/components/input/text_editor.rs`: existing ownership/binding pattern.
- `crates/rustui/src/view.rs`: retained entities, owner binding writer, subscriptions.
- `vendor/gpui-component-0.7.3/src/input/editor.rs`: native styled editor facade.
- `vendor/gpui-base-0.7.3/src/input/base/state.rs`: native construction, language,
  value, placeholder, and line-number APIs.

## Syntax highlighting boundary and separate dependency proposal

Current workspace dependency configuration is
`gpui-component = { version = "=0.7.3", default-features = false }`.
`vendor/gpui-component-0.7.3/src/highlighter/mod.rs:10-18` selects a factory that
returns `None` when `tree-sitter` is disabled. This adapter therefore exposes
language editing settings and native source editing without claiming syntax
coloring. It configures no language server and does not establish LSP support.

The following is a read-only dependency proposal, not an enabled feature or
approved dependency change:

- JSON highlighting: add `features = ["tree-sitter"]` to the existing pinned
  GPUI Components workspace dependency. This enables its optional native
  tree-sitter engine and JSON grammar.
- Rust plus JSON highlighting: use `features = ["tree-sitter-rust"]` instead;
  that feature includes `tree-sitter` and the Rust grammar. Broad grammar
  bundles are not required for these languages.
- Keep `gpui-component =0.7.3` and `gpui-pre =0.3.9` unchanged. The vendored
  Components manifest declares native optional requirements `tree-sitter
  ^0.26.13`, `tree-sitter-json ^0.24.8`, and `tree-sitter-rust ^0.24.0`.
- Concrete proposed new lock/vendor packages are `tree-sitter 0.26.13`,
  `tree-sitter-json 0.24.8`, `tree-sitter-language 0.1.9`, and
  `streaming-iterator 0.1.9`; add `tree-sitter-rust 0.24.0` for Rust. All five
  package families are absent from the inspected Cargo.lock and vendor tree.
  After approval, resolve/lock these versions and vendor their checksums and
  licenses reproducibly before building. No dependency source edit is needed.

Read-only crates.io dependency metadata for the stated versions confirms
that the engine additionally uses `cc ^1.2.48`, `regex ^1.11.3`,
`regex-syntax ^0.8.6`, and build-time `serde_json ^1.0.145` with
`preserve_order`; the inspected lock/vendor versions already satisfy these
requirements (`cc 1.8.0`, `regex 1.13.1`, `regex-syntax 0.8.11`,
`serde_json 1.0.154`, with indexmap already in its dependency graph).
Both grammars use existing `cc ^1.1` and the proposed `tree-sitter-language
^0.1`. The proposed 0.1.9 language and streaming-iterator packages have no
non-dev dependencies. The engine's default feature is `std`; optional bindgen
and wasm dependencies are not needed for this native configuration. Actual
Cargo resolution/build verification remains pending approval.

Grammar query strings for JSON and Rust are already included by the pinned
upstream source under `src/highlighter/languages/`; compiled grammar crates
would still need the separate feature/dependency additions above. Native
syntax rendering must be verified after those additions; a language string
alone cannot establish highlighting acceptance.

## Native acceptance evidence and remaining limits

Milestone 53 received bounded native editing acceptance in
`target/headstart/milestone53-code-editor-gui.json` using
`verification/M52 M55 Review.app`. The reviewed binary SHA-256 was
`05fabb23fc5780056a01adeca40eab671a2704a740ba428638aab531d027ab9a`.

The native monospace editor displayed line numbers 1–4 and exposed a named
“Source code” AX text entry area. Typing `let value = 1;` and appending
`// edit` produced a matching 22-character binding mirror. Changing the
language from Rust to JSON and turning line numbers off preserved the buffer
and removed the visible gutter. Native Undo and Redo changed the retained
buffer after those settings updates; exact undo coalescing was not asserted.
Readonly and disabled input were independently suppressed. An external value
replacement updated the disabled editor and binding mirror to a 32-character
JSON value. Removing the control removed the AX editor, and remount restored
that bound source.

The shared Milestones 52–55 accepted test log reports 160 passed and 0 failed
(`target/headstart/milestone52-55-tests-accepted.log`). The reuse report
confirms the successful build reused all-fresh `gpui-component 0.7.3`,
`gpui-pre 0.3.9`, and `gpui-pre-platform 0.3.9` artifacts
(`target/headstart/milestone52-55-reuse-report.json`).

Syntax highlighting remains unavailable pending approval of the parser
dependency feature; no syntax rendering is accepted. There is no LSP
integration, full editor accessibility or screen-reader acceptance, or
large-file performance result. Exact undo coalescing and native identity or
history continuity across removal/remount are not guaranteed.
# M409 — refresh retained CodeEditor binding
CodeEditor now routes retained input changes through the latest owner `Binding` after same-identity rerenders. Focused CodeEditor tests passed 5/5, including the latest-writer regression. Native editor interaction, GUI, cursor/undo, and full accessibility acceptance remain open. Evidence: `target/headstart/evidence/code-editor-latest-binding-writer-20261021.log`.
