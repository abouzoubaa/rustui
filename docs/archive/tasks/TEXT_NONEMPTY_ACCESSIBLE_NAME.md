# Text nonempty accessible names

`Text` rejects blank literal values, and `LocalizedText` rejects blank message IDs. A nonblank localized message such as `{name}` can resolve to blank when its supplied argument is empty. RustUI still renders the resolved content, but omits the explicit GPUI accessibility label when the resolved string is blank; it does not panic during rendering.

## Validation

- The focused regression supplies an empty `name` argument, confirms interpolation produces an empty string, and verifies the same label-selection helper used by rendering omits blank and whitespace-only labels while preserving nonblank labels.
- The initial focused test only verified a panic in a helper and did not prove safe rendering. The regression was corrected on 2026-10-21.
- The corrected focused RustUI test result is recorded in `target/headstart/evidence/m504-text-nonempty-accessible-name-20261021-rerun.log`.
- This is source-level evidence. Native accessibility-tree and screen-reader behavior remain open under [`TEXT_ACCESSIBILITY.md`](TEXT_ACCESSIBILITY.md).

Evidence: `target/headstart/evidence/m504-text-nonempty-accessible-name-20261021-rerun.log`.
