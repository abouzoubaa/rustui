# D1c Skeleton preview specimen

The compiled-preview gallery registers a signal-free Skeleton specimen
through the public RustUI API. Its schema exposes one editable `secondary`
Boolean (default `false`). The specimen uses fixed 160×20 logical-pixel
geometry, a 6-pixel corner radius, the accessible label `Loading content`,
and a generation-specific identity.

The visible Playground mount control is `Preview Skeleton specimen`. The
existing compiled-preview host and property inspector provide descriptor
editing and import/export.

Source implementation is recorded as M267. The locked/offline Playground
build passed. Native visual, progress-indicator role/name, keyboard/focus,
localization, platform, and VoiceOver acceptance remain open. Build output is
`target/headstart/evidence/m267-skeleton-preview-build-20261020.log`.

## M717 — Stage 9 descriptor regression

The compiled-preview specimen had no focused descriptor regression for its
editable `secondary` Boolean. The new test checks its schema/default, imports a
custom secondary descriptor, and verifies the normalized export preserves the
value. The focused locked/offline test passed; visual and platform acceptance
remain open.
