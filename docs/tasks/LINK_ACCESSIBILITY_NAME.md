# Link accessible-name validation

`Link` uses its visible label as the default accessible name. An explicit
`accessibility_label(...)` override must contain at least one non-whitespace
character. The builder checks the trimmed view and retains nonblank text
exactly as supplied; visible label and URL remain unchanged.

At render time, the resolved name must be nonblank. This permits an icon-only
link with an explicit name while preventing a blank visible label from becoming
an unnamed native control.

Focused source tests cover whitespace-only rejection and preservation of a
nonblank override, including surrounding whitespace, plus default-name
resolution and the required override for an empty visible label. They do not
establish native accessibility-tree, screen-reader, or cross-platform
acceptance.

## Evidence — M345

- `target/headstart/evidence/link-nonempty-accessible-name-m345-test.log`
- `target/headstart/evidence/link-nonempty-accessible-name-m345-check.log`
- `rustfmt --check --edition 2024 crates/rustui/src/components/basic/link.rs` passed.
- Scoped `git diff --check` passed.

## M612 — Localized Link accessible names

`Link::localized_accessibility_label(message_id)` resolves the accessible name
through the active RustUI catalog. Missing, blank, or key-echo values fall back
to the explicit literal name, then the visible link label. The builder rejects
blank message IDs and requires a nonblank fallback so icon-only links keep a
usable name when a translation is unavailable. The canonical Playground Link
story demonstrates the API; English, French, and Arabic runtime catalogs
provide the message.

Validation: the focused Link tests passed 12/12, the focused catalog contract
passed 1/1, locale validation passed, and the locked/offline Playground
all-targets check passed. Evidence:
`target/headstart/evidence/m612-link-localized-name-tests-20261021.log`,
`target/headstart/evidence/m612-link-catalog-contract-20261021.log`,
`target/headstart/evidence/m612-link-locale-validation-20261021.log`, and
`target/headstart/evidence/m612-link-playground-check-20261021.log`.

Native AX, live locale switching, VoiceOver announcements, and cross-platform
acceptance remain open.
