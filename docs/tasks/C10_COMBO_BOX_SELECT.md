# C10: ComboBox Select adapter

`ComboBox` keeps its RustUI API and retained identity while using the pinned
GPUI Components `SelectState<SearchableVec<String>>` and `Select` view. Search
is always enabled. The visible placeholder and accessible name both use the
existing label; the existing search placeholder, appearance, and disabled
settings are forwarded to the native control.

The retained selection is initialized from the binding, updated when options
or externally controlled selection change, and written back only on
`SelectEvent::Confirm(Some(value))`. `Confirm(None)` leaves the string binding
unchanged. The identity is also forwarded as the native Select ID.

## M77–M78 popup replacement regression

The M77 before-fix probe found a native popup reconciliation defect. After
selecting Skia and replacing the options with `[Skia, Metal, CoreGraphics]`, the
binding and closed trigger still showed Skia, but the open popup selected
CoreGraphics at the old source index 2. Escape and reopening reproduced the
mismatch. The same probe observed Metal selection updating the owner, disabled
state blocking popup opening, and an owner update to Skia changing the disabled
trigger. Evidence: `target/headstart/milestone77-combobox-before-fix.json` and
`target/headstart/milestone77-combobox-popup-mismatch.png`.

The M78 build fixed the regression: after selecting Skia and replacing the
options, the popup selected the first row, Skia. Native checks observed Metal
selection updating the binding, disabled state blocking the popup, an external
owner update to Skia changing the disabled trigger, and the re-enabled popup
opening with Skia selected. Removing the selected option left the owner value
as CoreGraphics and the native popup blank; restoring options and selecting
Skia recovered. This missing-option observation adds no normalization contract.
Evidence: `target/headstart/milestone78-combobox-gui.json`,
`target/headstart/milestone78-combobox-popup-fixed.png`, and the packaged binary
record `target/headstart/verification/milestone78-navigation-combo-box-review.json`
(binary SHA-256
`df0d1e92f50d116aef04478fe1363c77d0444d14f1e5a0aa25596a1629d71025`).

The workspace run passed 252 tests across 12 binaries; the final all-targets
check and Playground build exited 0. See
`target/headstart/milestone78-workspace-tests.log`,
`target/headstart/milestone78-final-all-targets.log`, and
`target/headstart/milestone78-playground-build.log`.

`ComboBox` receives the corrected shared Select reconciliation path. `Picker`
uses the shared source but was not separately checked in the native app. Full
keyboard/focus, accessibility, and broader visual acceptance remain pending.

## Executed native acceptance (M28)

The native trigger exposes Rendering engine and selected value GPUI. Filtering
to Skia leaves one selectable result; Return confirms it, closes the popup, and
updates the trigger's AX value to Skia. Nonmatching Qt returned no selectable
results. Evidence: `target/headstart/milestone28-combobox-gui.json`; build log:
`target/headstart/milestone27-28-native-build.log`. At M28, disabled behavior
and external options replacement remained separate native cases; the bounded
M78 follow-up above now covers those cases. No upstream patch or dependency
change was used.

## Picker native acceptance (2026-10-19)

The canonical Playground Inputs story now includes an independent Enabled
toggle, a Kotlin owner-value setter, and a visible owner-value readout for
Picker. Using `dev.rustui.playground`, selecting Swift changed the readout and
trigger to Swift. Disabling Picker prevented its popup from opening or changing
the value. Setting Kotlin externally updated the readout and disabled trigger;
re-enabling opened the popup with Kotlin selected. Evidence:
`target/headstart/evidence/picker-acceptance-20261019.json`.
