# Retained input identity validation — M469

All nine input builders that define an explicit `.id(...)` method reject empty or whitespace-only IDs before using them for retained state: `TextField`, `TextEditor`, `Picker`, `ComboBox`, `Toggle`, `Checkbox`, `Radio`, `Slider`, and `Stepper`. Valid caller IDs are preserved exactly; validation checks trimmed emptiness without trimming or rewriting the identity. `SearchField` and `SecureField` return the shared `TextFieldView`, so they inherit its validation.

The locked/offline RustUI identity-filtered suite passed 52/52 tests, including blank-ID rejection for the seven additional builders and existing empty/whitespace and verbatim-preservation cases for TextField and TextEditor. Targeted rustfmt and `git diff --check` passed. Native keyboard, focus, IME, screen-reader, and GUI acceptance remain separate gates.

Evidence: `target/headstart/evidence/m469-expanded-input-identity-verified-20261021.log`.

## M507 follow-up — ColorPicker identity

`ColorPickerView::id(...)` now rejects empty and whitespace-only retained-state
identities while preserving the caller's nonblank value verbatim. Focused
rejection and preservation regressions are recorded in
`target/headstart/evidence/m507-color-picker-identity-validation-20261021.log`.
This extends the M469 inventory from nine input builders; it does not change the
historical M469 test result above.

After M507, the complete RustUI library unit suite passed 575/575:
`target/headstart/evidence/rustui-lib-regression-after-m507-20261021.log`.

## M508 follow-up — DatePicker identity

`DatePickerView::id(...)` now rejects empty and whitespace-only retained-state
identities while preserving valid IDs verbatim. Focused rejection and
preservation tests plus the full RustUI library suite are recorded in
`target/headstart/evidence/m508-datepicker-identity-validation-20261021.log`
and `target/headstart/evidence/rustui-lib-regression-after-m508-20261021.log`.

## M526 follow-up — DatePicker sibling identity

Implicit DatePicker identities now use `RenderContext::next_implicit_retained_identity`, separating same-label controls by owner sibling slot while keeping each slot stable across rerenders. Explicit `.id(...)` values still bypass fallback allocation. The focused DatePicker suite passed 8/8 in `target/headstart/evidence/m526-datepicker-sibling-identity-tests-20261021.log`. Native two-picker binding isolation and trigger/popup accessibility acceptance remain open.

## M531 Picker-family sibling identity

Picker, ComboBox, and ColorPicker use owner-scoped implicit sibling slots for
retained state; explicit `.id(...)` values remain unchanged. Focused tests
verify same-label sibling distinction and stable slot identities across
rerenders. All 3 focused tests passed in
`target/headstart/evidence/m531-picker-family-sibling-identity-tests-20261021.log`.
Native binding isolation and accessibility acceptance remain open.
