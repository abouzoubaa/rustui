# D1c Label preview specimen

The compiled-preview gallery registers an opt-in `Label` specimen through the
public RustUI API. Its schema exposes `text` (default `Search`, 1–80
characters), `secondary` (default `⌘K`, up to 80 characters), and `vibrant`
(default `false`). The specimen emits no events and adds no icon asset
dependency. Whitespace-only primary text renders a safe `Label` fallback
because `LabelView::new` rejects blank accessible text.

The visible Playground mount control is `Preview Label specimen`. The existing
compiled-preview host and property inspector provide the descriptor editing
and import/export path.

Source implementation is recorded as M262. The locked/offline Playground
build passed. No focused regression or native GUI run is claimed here. Native
appearance, accessibility-tree, keyboard, localization, and VoiceOver
acceptance remain open. Build output is
`target/headstart/evidence/m262-label-preview-build-20261020.log`.

## M719 — Stage 9 descriptor regression

The compiled-preview specimen had no focused descriptor regression for its
editable `text`, `secondary`, and `vibrant` properties. The new test imports
custom values for all three and verifies normalized export preserves them. The
focused locked/offline test passed; native visual and accessibility acceptance
remain open.
