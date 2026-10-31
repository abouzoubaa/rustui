# M336 — Stage 9 / D1c LocalizedText preview specimen

The compiled preview gallery registers an opt-in `LocalizedText` specimen through
RustUI's public API. Its bounded `message_id` property offers only
`rustui-action-close` and `rustui-action-confirm`; both keys resolve in the
Playground's English, French, and Arabic catalogs through the same
`Catalogs::resolve` path used by the active localization context. The specimen
has no events or action targets.

The regression verifies that the mounted descriptor's `message_id` is passed
through the specimen factory helper, updates that property through generation-
stamped binding feedback, and preserves the selected key through descriptor
export/import. It checks the exact close/confirm translations in all three
catalogs while the mounted preview stamp and source message ID stay unchanged.

Source inspection confirms locale refresh without remount: `ComponentHost`
subscribes to `ActiveLocalization` changes and notifies; its next render creates
a fresh `RenderContext` from the active localization. The preview registry
factory captures the mounted descriptor snapshot, not the locale, and the
`LocalizedText` view resolves its message ID from the current render context.
The preview key remains based on preview ID and generation, so a locale change
can refresh the translated label while retaining the preview generation. The
source regression checks catalog output and unchanged mount identity; it does
not execute a GPUI global-change/render cycle.

Validation evidence:

- `target/headstart/evidence/d1c-localized-text-preview-locale-refresh-m336-20261021.log` — `localized_text_specimen_keeps_mount_stamp_while_catalog_resolution_changes` passed (1/1; 65 filtered out).
- `target/headstart/evidence/d1c-localized-text-preview-tests-m336-20261021.log` — original focused regression passed (1/1).
- `target/headstart/evidence/d1c-localized-text-preview-check-m336-20261021.log` — isolated locked/offline Playground all-targets check passed after the shared TextField import correction.
- Rustfmt check and scoped `git diff --check` passed after the source regression update. Native GUI/accessibility acceptance was not run.
