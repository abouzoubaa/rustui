# D1c PropertyInspector preview specimen

## Contract

Add an opt-in, signal-free D1c specimen for the implemented public
`PropertyInspector` API. Its compiled factory presents one text row, one toggle
row, and one integer row, using existing gallery-owned bindings. The descriptor
can edit each bounded row label and whether the inspector is disabled. The row
IDs, editor kinds, options, and bindings remain local to the compiled factory;
the descriptor does not serialize closures or runtime state. The specimen has no
events or actions and uses the existing preview descriptor codec.

## M361 — Stage 9 / D1c PropertyInspector specimen

The schema limits each nonblank row label to 80 characters and exposes a
Boolean disabled property. The focused round-trip test passed 1/1, and the locked/offline Playground
all-targets check passed. Native editing, visual layout, keyboard, focus,
accessibility, localization, and cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/d1c-property-inspector-preview-tests-m361-followup-20261021.log`
and `target/headstart/evidence/d1c-property-inspector-preview-check-m361-20261021.log`.
No source outside the D1c gallery was changed for this specimen.
## M444 — PropertyInspector mount action

The compiled gallery’s `Preview PropertyInspector specimen` button is wired to
the mount handler. The focused test invokes that handler, confirms the specimen
is mounted, and passes its bounded-label and disabled-state round-trip checks
(1/1: `target/headstart/evidence/property-inspector-gallery-m444-test.log`).
The test does not click the button in a running Playground; native interaction,
visual, keyboard, and accessibility acceptance remain open.

## DatePicker inspector write validation

The compiled DatePicker specimen's string property editor accepts an empty
value or a real canonical `YYYY-MM-DD` date with a four-digit year from 0001
through 9999. Its binding rejects malformed and impossible dates before
updating preview state. The focused regression passed 1/1, covering malformed
and impossible writes, an accepted canonical date, and preservation of the
previous value after rejection. The locked/offline Playground all-targets check
passed. Evidence:
`target/headstart/evidence/d1c-date-inspector-canonical-write-20261021.log`
and `target/headstart/evidence/d1c-date-inspector-canonical-write-check-20261021.log`.
This verifies the compiled inspector binding; native editing, visual layout,
keyboard, focus, accessibility, localization, and cross-platform acceptance
remain open.

## M484 DatePicker rejected-write announcement (2026-10-21)

The inspector status line is a stable polite `Status` live region, but its
announced value is populated only when a DatePicker date write is rejected for
being noncanonical or impossible. Ordinary inspector status changes leave the
live-region value empty, avoiding announcements for routine edits. The focused
DatePicker regression verifies that a rejected write exposes the error value,
preserves the mounted property, and a subsequent accepted write clears the
announcement. The focused test passed 1/1 and the locked/offline Playground
all-targets check passed. Evidence:
`target/headstart/evidence/m484-datepicker-live-rejection-test-20261021.log`
and `target/headstart/evidence/m484-datepicker-live-rejection-check-20261021.log`.
Native inspector editing, visible error feedback, keyboard/focus, VoiceOver,
localization, and cross-platform acceptance remain open.

## M488 — Stage 9 / announce generic preview-inspector write rejections

The existing polite Status live region now announces rejected text and numeric
inspector writes as well as DatePicker date rejections. Blank required text and
out-of-range numeric values leave the mounted preview value unchanged; an
accepted numeric write clears the announcement. The compiled-preview test module
passed 63/63 and the locked/offline Playground all-targets check passed. Native
VoiceOver speech remains unverified. Evidence:
`target/headstart/evidence/m488-preview-inspector-module-tests-20261021.log` and
`target/headstart/evidence/m488-preview-inspector-rejection-check-20261021.log`.
