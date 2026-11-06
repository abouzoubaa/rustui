# Stage 9 / D1c HStack preview specimen

The canonical compiled-preview gallery now registers an opt-in specimen for
the public RustUI `HStack` layout. Its schema exposes one editable numeric
`spacing` property bounded to 0–32 logical pixels, defaulting to 12. The
compiled-local factory keeps a centered, three-label child structure fixed and
maps the bounded value to `HStack::spacing`; no child structure or callback is
serialized.

The regression checks the schema bounds and event-free contract, then changes
spacing to 24.5, exports and imports the descriptor, and verifies the mounted
value survives the round trip. The first focused run found that gallery mount
initialization placed new specimens on a fallback path that inserts the shared
`amount` property. HStack was added to the existing schema-default path; the
rerun passed.

Validation:

- `./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::hstack_specimen_round_trips_bounded_spacing_without_events --locked --offline` — 1 passed. The initial failure is preserved in `target/headstart/evidence/d1c-hstack-specimen-initial-failure-20261021.log`; the passing rerun is `target/headstart/evidence/d1c-hstack-specimen-test-20261021.log`.
- `./scripts/cargo-headstart check -p rustui-playground --all-targets --locked --offline` — exited 0, with the existing unused `mount_property_inspector_specimen` warning. Log: `target/headstart/evidence/d1c-hstack-specimen-check-20261021.log`.
- `rustfmt --edition 2024 apps/rustui-playground/src/stories/compiled_preview_gallery.rs` and scoped `git diff --check` passed.

No GUI run was performed for this specimen. Native layout appearance, keyboard
and screen-reader behavior, localization/RTL, and cross-platform acceptance
remain open.
