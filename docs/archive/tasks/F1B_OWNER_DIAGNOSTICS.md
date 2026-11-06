# F1B: Child Owner Mismatch Diagnostics

Action dispatch with an owner of the wrong concrete component type is a no-op and
returns `Invalidation::None` (M530). Binding mismatch behavior is documented in
`R1_BINDING_UPDATES.md`. `Binding::read_from` remains an optional type probe, and
an update closure that reports no change still returns `false` without writing.

These diagnostics check only the owner's concrete type. Actions and bindings are
owner-relative, type-erased lenses, so two instances of the same component type
cannot be distinguished. This baseline also has no API for targeting a parent
component from a child. Child actions and bindings must therefore be routed through
the correct component host; instance identity and explicit parent targeting remain
unsupported.

## M712 — Stage 3 / correct Action owner-mismatch wording

Corrected the stale Action dispatch claim to match M530 and current source:
wrong-owner dispatch returns `Invalidation::None` without mutating the target.
Documentation-only; no source or runtime behavior changed. Scoped diff check
passed. Binding mismatch behavior remains documented in `R1_BINDING_UPDATES.md`.
