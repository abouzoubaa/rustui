# M1037 — Serialize Navigation semantics name

## Scope

Existing semantics checks exercised label application directly, but did not
pass through RustUI's production semantic scoped element wrapper.

## Change

The new regression constructs `semantic_scoped_element` with Navigation role
and a literal accessible name. It downcasts the stored GPUI element, verifies
the exposed role, and writes the name through GPUI's element accessibility
hook into an AccessKit node.

## Verification

`./scripts/cargo-headstart test -p rustui --lib runtime::scoped_element::scope_tests::semantic_scoped_element_serializes_navigation_accessible_name --locked --offline`

Result: 1 passed, 0 failed.

This is source-level node serialization evidence. Native accessibility-tree
exposure, VoiceOver speech, keyboard navigation, and cross-platform acceptance
remain open.
