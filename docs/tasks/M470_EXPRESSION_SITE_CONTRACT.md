# M470 — Expression-site catalog contract validation

Cross-catalog validation previously unioned branch keys for every plural or select expression that used the same argument and expression kind. Two repeated `select` sites could therefore swap distinct keys and still validate, even though runtime interpolation at either site could fall through to `other`.

`MessageContract` now retains recursive expression sites, keyed by argument, kind, and same-kind sibling ordinal. Select sites require exact branch-key parity and recursively matching nested contracts. Plural sites require exact-number parity, compare category bodies with shared labels directly, and compare canonical nested contract shapes across categories so locales can add plural categories when they reuse existing structures. Novel nested structures in extra plural categories remain mismatches.

Regression coverage includes swapped repeated sibling select keys, swapped nested select keys under shared plural categories, valid additional Arabic plural categories reusing existing nested shapes, and rejection of a novel nested selector shape in an extra category. The repeated-select case also verifies runtime fallback behavior. Locked/offline checks passed: `rustui-i18n` 33 unit tests and 3 catalog-contract tests; RustUI localization filter 10/10. Targeted rustfmt and `git diff --check` passed.

Evidence:

- `target/headstart/evidence/m470-expression-site-contract-tests-20261021-final.log`
- `target/headstart/evidence/m470-rustui-localization-tests-20261021.log`

This establishes source and package-test coverage. Native locale switching, RTL layout, and screen-reader acceptance remain separate gates.
