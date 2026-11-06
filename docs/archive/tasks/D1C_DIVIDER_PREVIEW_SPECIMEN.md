# D1c Divider preview specimen

The compiled-preview gallery registers a signal-free Divider specimen through
the public RustUI API. Its schema exposes one editable `vertical` Boolean
(default `false`). Horizontal mode places the divider between `Above` and
`Below` text in a `VStack`; vertical mode places it between `Left` and `Right`
text in an `HStack`. The specimen uses no new dependency, asset, or event.

The visible Playground mount control is `Preview Divider specimen`. The
existing compiled-preview host and property inspector provide descriptor
editing and import/export.

Source implementation is recorded as M266. The locked/offline Playground
build passed. Native orientation, sizing, keyboard/focus, localization, and
cross-platform acceptance remain open. Build output is
`target/headstart/evidence/m266-divider-preview-build-20261020.log`.

## M714 — Stage 9 descriptor regression

The compiled-preview specimen had no focused descriptor regression for its
editable `vertical` Boolean. The new test checks its schema/default, imports a
custom vertical descriptor, and verifies the normalized export preserves the
value. The focused locked/offline test passed; visual and platform acceptance
remain open.
