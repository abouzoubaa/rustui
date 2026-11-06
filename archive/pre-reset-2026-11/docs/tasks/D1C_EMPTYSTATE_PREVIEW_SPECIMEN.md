# D1c EmptyState preview specimen

The canonical compiled-preview gallery includes an opt-in
`empty-state-specimen` backed by the public RustUI `EmptyState` API. It has a
required nonblank title bounded to 1–120 characters and optional detail bounded
to 240 characters. Whitespace-only detail is omitted from the rendered view.
The descriptor remains signal-only with no events.

The preview wraps `EmptyState` in the public fixed-size `Frame` API so the
component's `size_full()` root is bounded to a 320 × 180 preview rectangle and
does not take space from the adjacent schema inspector.

A focused regression covers schema bounds, signal-only registration, blank-title
import rejection without replacing the mounted preview, omitted blank detail,
and descriptor export/import round-trip. The isolated locked/offline
Playground library suite passed 41/41, including this regression. Evidence:
`target/headstart/evidence/roadmap-validation-m251-playground-lib.log`. No GUI or
native accessibility acceptance was run.
