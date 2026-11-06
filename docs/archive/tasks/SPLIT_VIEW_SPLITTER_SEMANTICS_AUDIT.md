# SplitView splitter semantics source audit

The bounded Stage 8 source pass found the requested divider semantics already
implemented. This records current source behavior, not a new native acceptance
result. See `SPLIT_VIEW_IMPLEMENTATION.md` for the existing implementation and
native evidence limits.

- `crates/rustui/src/layout/split_view.rs:333` obtains current/minimum/maximum
  values from `divider_aria_values`. That helper clamps the current measured size
  (or fallback) to the same effective range (`:213`).
- The divider has a panel-keyed ID, is focusable and a tab stop, has
  `Role::Splitter`, and uses the localized resize-divider name (`:335-350`).
  Horizontal panel layout gives the divider vertical orientation; vertical panel
  layout gives it horizontal orientation (`:350-353`).
- It exposes numeric current/minimum/maximum size and a 10-point step
  (`:354-357`). The pinned GPUI `Interactivity::write_a11y_info` writes these
  fields and orientation into AccessKit nodes
  (`vendor/gpui-pre-0.3.9/src/elements/div.rs:3566-3585`).
- `divider_keyboard_size` maps Left/Right to horizontal panel resizing and Up/Down
  to vertical panel resizing in 10-point steps; other directions do not resize,
  and results clamp to the range (`split_view.rs:163`). The key handler ignores
  character-input, IME, and modified events, then calls the native
  `ResizableState::resize_panel` API (`:358-378`).
- Components publicly re-exports `ResizableState` from GPUI Base
  (`vendor/gpui-component-0.7.3/src/resizable.rs:19`). The public implementation
  delegates to the native handle resize path and emits the resize completion
  notification (`vendor/gpui-base-0.7.3/src/resizable/mod.rs:69-89`). RustUI does
  not need to replace the native resizing engine to support its Arrow keys.

Existing focused regressions cover the accessible-name fallback (`split_view.rs:408`),
axis-specific Arrow keys and clamping (`:466`), numeric values (`:505`), and
container/sibling feasible bounds (`:518`). These tests were inspected, not rerun
for this documentation-only audit.

No missing RustUI-owned public behavior was established in this scope. Native
screen-reader announcements and current/minimum/maximum value exposure after
keyboard resizing remain acceptance questions; source inspection and helper
tests cannot establish those outcomes. No code, vendor source, dependency
revision, or central ledger was changed, and no GUI probe was run in this pass.
