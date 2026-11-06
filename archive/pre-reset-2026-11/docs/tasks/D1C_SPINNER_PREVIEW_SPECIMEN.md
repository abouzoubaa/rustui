# D1c Spinner preview specimen

The compiled-preview gallery registers an opt-in, signal-free Spinner through
the public RustUI API. Its schema exposes one editable
`accessibility_label` property (default `Loading`, 1–80 characters). The
factory assigns a generation-specific identity. Whitespace-only values use
`Loading` as a safe fallback because `SpinnerView::accessibility_label`
rejects blank names.

The visible Playground mount control is `Preview Spinner specimen`. The
existing compiled-preview host and property inspector provide descriptor
editing and import/export.

Source implementation is recorded as M265. M268 registers the already-locked
GPUI Kit default asset bundle in the canonical Playground, including the
Loader SVG used by the pinned Spinner. The locked/offline M265 Playground
build passed. Native visual, accessibility-tree role/name, keyboard/focus,
localization, platform, and VoiceOver acceptance remain open. Build output is
`target/headstart/evidence/m265-spinner-preview-build-20261020.log`.


M268 Playground asset registration compiles with the locked/offline
all-targets check at `target/headstart/evidence/m268-icon-assets-playground-check-20261020.log`. This is compile evidence, not native rendering acceptance.

## M713 — Stage 9 descriptor regression

The catalog coverage matrix includes Spinner, but the compiled-preview gallery
had no focused descriptor regression for its editable accessibility label. The
new regression imports and exports a custom label, then confirms a whitespace
value accepted by the schema's one-character minimum remains round-trippable.
The factory's existing `Loading` fallback handles that schema-valid value
before calling the public Spinner builder, which rejects blank labels. The
focused locked/offline test passed. No visual or accessibility acceptance is
claimed.
