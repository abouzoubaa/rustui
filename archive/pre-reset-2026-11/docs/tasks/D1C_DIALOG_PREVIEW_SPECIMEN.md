# D1c Dialog preview specimen

The canonical compiled-preview gallery includes an opt-in public RustUI
`Dialog` specimen. Its descriptor serializes the editable trigger label; the
compiled factory owns the fixed title and body. Confirm and cancel are separate
signal-only events with empty payload schemas, each resolved through the
existing allowlisted owner action. No view, closure, or callback is serialized.

The focused regression covers the trigger schema, empty confirm/cancel payloads,
allowlisted dispatch for both signals, stamp-checked trigger feedback, and
descriptor export/import round-trip. Native dialog opening, confirmation,
cancellation, keyboard behavior, accessibility, localization, and
cross-platform behavior remain unverified until a GUI acceptance run.

## Validation

- Focused test passed 1/1 using the locked/offline isolated target. Log:
  `target/headstart/evidence/d1c-dialog-specimen-tests-20261020.log`.
- The isolated locked/offline Playground all-targets check passed after the
  concurrent DataGrid source fix. The first attempt surfaced missing
  `StatefulInteractiveElement` imports in `table.rs`; that source was corrected
  before this successful rerun. Log:
  `target/headstart/evidence/d1c-dialog-specimen-check-20261020.log`.
