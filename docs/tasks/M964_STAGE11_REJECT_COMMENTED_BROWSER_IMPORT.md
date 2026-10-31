# M964 — Reject commented-out browser module imports

Stage 11 / WebAssembly browser packaging.

The package guard no longer accepts a commented-out
`./rustui_playground.js` import as evidence that the browser shell loads the
generated module.

The focused packaging harness rejected the commented import, preserved the
previous bundle, and confirmed staging cleanup. The harness also retains the
missing-module and valid-shell cases. Shell syntax and scoped whitespace checks
passed.

This milestone covers package readiness only. Actual browser startup,
rendering, input, asset loading, and accessibility acceptance remain open.
