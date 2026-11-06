# M1164 — Stage 9 / announce out-of-range Stepper inspector values

The compiled-preview inspector's `IntegerValue` editor accepts `i64` drafts,
while the Stepper specimen is backed by an `i32`. Entering a value outside the
Stepper control's range leaves the mounted preview unchanged and sets the
status to `Stepper value must fit its control`, but the polite live-region
filter omitted that message. The filter now includes the emitted status.

The focused regression failed before the fix because the live-region value was
`None`; it passed afterward and verifies the preview stamp and properties do
not change. Native screen-reader acceptance remains open.
