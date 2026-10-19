# F1B: Child Owner Mismatch Diagnostics

Action dispatch and binding writes/updates now panic with a clear diagnostic when
given an owner of the wrong concrete component type. `Binding::read_from` remains
an optional type probe, and an update closure that reports no change still returns
`false` without writing.

These diagnostics check only the owner's concrete type. Actions and bindings are
owner-relative, type-erased lenses, so two instances of the same component type
cannot be distinguished. This baseline also has no API for targeting a parent
component from a child. Child actions and bindings must therefore be routed through
the correct component host; instance identity and explicit parent targeting remain
unsupported.
