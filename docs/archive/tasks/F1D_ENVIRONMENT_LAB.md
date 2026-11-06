# F1D Environment Lab demo

## Scope

Demonstrate RustUI's typed, scoped `Environment(value, view)` API in the existing
Layout story. The parent owns a `DemoDisplayMode` value and wraps the retained
Alpha/Beta child stack in that environment. Each child reads the optional typed
value with `EnvironmentReader` and displays the mode, including a fallback when
no value is in scope.

This setting is a RustUI environment demo. It does not represent a theme, locale,
operating-system setting, or second state runtime.

## Behavior

- The parent control switches between compact and detailed values.
- Both retained children receive the same value on the next parent render.
- The environment only changes the child description; title props and child
  counter and notes state remain independent.
- Reordering and removing children continue to use the existing keyed/component
  behavior.
- `EnvironmentReader` samples the scoped value when the parent renders. This
  demo makes no claim of asynchronous or fine-grained observation.

## Files

The demo uses `apps/rustui-lab/src/main.rs`, `canvas.rs`,
`child_component.rs`, and `code_preview.rs`. The catalog continues to describe
this as the Layout story.

## Verification boundary

No tests, Cargo commands, GUI checks, dependency changes, vendored edits, or Git
operations are part of this implementation task. Formatting is limited to the
assigned Rust files with `rustfmt --skip-children`.
