# D1c Tooltip preview specimen

The compiled-preview gallery registers a signal-free Tooltip specimen through
the existing public RustUI API. Its schema exposes `text` (default `More
information`, 1–120 characters) and child `label` (default `Help`, 1–80
characters). The compiled child is `Text`; the tooltip wrapper receives a
generation-specific stable identity. Tooltip text supplements the child and
does not replace its accessible name.

The visible Playground mount control is `Preview Tooltip specimen`. The
existing compiled-preview host and property inspector provide the descriptor
editing and import/export path.

Source implementation is recorded as M263. The locked/offline Playground
build passed; no focused regression or native GUI run is claimed. Native hover,
dismissal, keyboard/focus, accessibility-tree, VoiceOver, localization, and
cross-platform acceptance remain open. Build output is
`target/headstart/evidence/m263-tooltip-preview-build-20261020.log`.


M709 added a focused import regression for whitespace-only Tooltip text. The
schema's minimum length accepts whitespace, so import rejects blank text at the
component-contract boundary and preserves the mounted snapshot. The test also
exercises the registered specimen mount path; generic gallery state now seeds
only properties declared by the registered component schema, without replacing
values set by component-specific setup. The focused locked/offline unit test
passed. No native GUI or broader acceptance is claimed.
