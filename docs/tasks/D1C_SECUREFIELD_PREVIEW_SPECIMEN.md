# D1c SecureField preview specimen

The canonical compiled-preview gallery includes an opt-in public RustUI
`SecureField` with one serializable `placeholder` string. Its accessible name is
the fixed, nonblank `Preview secure input` label. The field value lives only in
an ephemeral gallery binding guarded by the mounted preview stamp; it is not a
schema property, preview document value, descriptor field, status message, or
owner summary. Remount and teardown clear the ephemeral value.

The descriptor schema has no events under the selected signal-only contract.
Import/export round-trips the placeholder configuration while discarding any
ephemeral field value. The focused test types a test-only sentinel, verifies it
is absent from the exported descriptor and status, imports configuration, and
confirms the old-generation binding is rejected.

## Validation

Focused test and isolated locked/offline Playground all-targets check:

- `target/headstart/evidence/d1c-securefield-preview-tests-20261020.log`
- `target/headstart/evidence/d1c-securefield-preview-check-20261020.log`

These checks establish source and compilation behavior only. No GUI or data
privacy acceptance was run; native masking, reveal behavior, accessibility,
locale/theme, and cross-platform behavior remain open.
