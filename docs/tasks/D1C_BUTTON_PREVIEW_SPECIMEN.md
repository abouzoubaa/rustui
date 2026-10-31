# D1c Button preview specimen

The compiled preview gallery registers an opt-in Button specimen through the
public RustUI `Button` API. Its descriptor contains only the editable string
`label` and Boolean `enabled` inputs. The `activate` event has an empty payload
schema and resolves through the existing allowlisted `increment` action. The
factory creates the Button with a generation-scoped identity and a signal
stamped from the mounted preview. Property edits use the existing revision
checked binding feedback path; export and import use the existing descriptor
codec. No event payload or descriptor-format changes were made.

The focused test checks the schema, empty event payload, accepted label and
enabled feedback, rejection of the pre-edit signal after the revision advances,
action dispatch after descriptor import, and round-trip of both values.

Validation on 2026-10-20:

- `CARGO_TARGET_DIR=target/headstart/d1c-button-specimen-validation ./scripts/cargo-headstart test --quiet -p rustui-playground --lib stories::compiled_preview_gallery::tests:: --locked --offline` — passed, 10 gallery tests, 0 failed, including the Button specimen.
- `CARGO_TARGET_DIR=target/headstart/d1c-button-specimen-validation ./scripts/cargo-headstart check --quiet -p rustui-playground --all-targets --locked --offline` — passed. Existing warnings remain for `THEME_CATALOG_LEN` and `table_binding`.
- `rustfmt --check --edition 2024 apps/rustui-playground/src/stories/compiled_preview_gallery.rs` — passed.
- `git diff --check` — passed.
- Corrected adjacent gallery test attributes: the TextField test had a duplicate `#[test]`, while the Stepper test lacked one. The gallery suite now runs both.
- An earlier focused-test attempt using the reused `target/headstart/localization-preview-check` artifacts hit the experimental Headstart incremental compiler ICE in `serde_core` (`optimized_mir` unstable fingerprint). The isolated target run above passed without cleaning shared artifacts.

No GUI acceptance was run; native appearance, pointer and keyboard behavior,
accessibility, locale, RTL, and cross-platform behavior remain unverified for
this specimen.
