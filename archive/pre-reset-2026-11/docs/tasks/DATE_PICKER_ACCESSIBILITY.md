# DatePicker accessible name

## Current implementation status (2026-10-21)

RustUI now names the focused DatePicker trigger with
`DatePickerView::accessibility_label(...)` or the localized
`DatePickerView::localized_accessibility_label(...)`; when no override is
supplied, a nonblank visible label is used as the stable name even after a date
is selected. A localized name retains the explicit name when supplied, or the
visible label otherwise, as its fallback. Whitespace-only literal names and
localization keys are rejected, and a blank visible label does not create an
implicit name. The canonical Playground Input story uses the visible-label
fallback; the compiled-preview DatePicker specimen exposes an editable
explicit-name property.

This works in the current workspace because the approved, reproducible patch
`patches/approved/gpui-component-date-picker-accessibility-v1.patch` is applied
to the pinned GPUI Components 0.7.3 source. No additional vendor or upstream
change is needed for this implemented API. A stock, unpatched 0.7.3 package
does not expose the internal focused trigger, so RustUI cannot set its name
through the public component builder alone.

## Historical limitation before M235

The RustUI DatePicker forwards its visible empty-state label through the pinned
GPUI Components `.placeholder(...)` builder. That does not name the focused
control: the component's renderer creates the ComboBox trigger internally with
`gpui_base::DatePicker::new(...)`, sets its role and selected-date value, and
does not apply an accessible label.

The pinned public GPUI Components `DatePicker` builder has no `aria_label`
method or accessible-label field. The lockfile pins `gpui-component` 0.7.3 and
`gpui-base` 0.7.3. In
`vendor/gpui-component-0.7.3/src/time/date_picker.rs`, the public element stores
`placeholder` but no accessible label; its `RenderOnce` implementation creates
`BaseDatePicker::new(self.id, &state.focus_handle)`, applies `.open(...)`, and
adds `.aria_value(...)` only when a date is selected. It does not call
`.aria_label(...)`. In
`vendor/gpui-base-0.7.3/src/date_picker.rs`, the base renderer supplies
`Role::ComboBox`, expanded state, and the focus handle. The base `DatePicker`
implements GPUI's `StatefulInteractiveElement`, which provides `.aria_label(...)`,
but that base instance is constructed inside the Components renderer and is not
exposed by its public `DatePicker` builder. RustUI receives only the public
Components element, so it cannot apply that trait method to the focused root. A
wrapper label would name a separate node and would not fix the focused ComboBox.

## Historical audit outcome before M235

There is no minimal wrapper-level RustUI fix within the pinned APIs. The proposed patch at
`patches/proposed/gpui-component-date-picker-accessibility.patch` sketches the
needed upstream builder and forwarding change, but it is not an approved,
applied patch and vendored source must remain untouched. Reopen implementation
when that capability is approved and made reproducibly available to this
workspace; then expose the label from the RustUI adapter and add focused source
and native accessibility acceptance.

A RustUI implementation that bypasses the Components `DatePicker` and rebuilds
its trigger and popup around `gpui_base::DatePicker` could call `.aria_label(...)`,
but it would replace the existing date-picker renderer and lifecycle. The pinned
`DatePickerState` keeps popup-open and toggle methods private, so that route would
also need a separate popup state and custom selection wiring; it is not a
wrapper-only fix and would duplicate upstream behavior.

## Current upstream capability

The approved patch adds an optional accessible-label builder to the pinned
Components DatePicker and forwards it to the existing focused
`gpui_base::DatePicker` ComboBox root. Cargo applies the versioned patch through
the workspace's local package override. Keep the patch and source hashes
aligned; do not edit the vendored package outside this reproducible patch
process. RustUI passes its visible-label fallback or explicit override through
that builder.

## Acceptance evidence still needed

- Source integration is compiled against the approved capability: the focused
  RustUI suite and locked/offline Playground all-targets check pass. See
  `target/headstart/evidence/date-picker-accessibility-capability-audit-20261021.log`.
- Inspect the native accessibility tree and verify the focused trigger exposes
  the ComboBox role and the expected label, both with an empty date and a
  selected date.
- Verify the selected date remains the accessible value and confirm the name
  is announced by VoiceOver. A source-level builder check alone is not native
  accessibility acceptance.
- Inspect the open calendar popup separately; naming the trigger does not
  establish accessible names or announcements for popup contents.


## Source audit evidence (2026-10-20)

The pinned-source audit and exact limits are recorded in `target/headstart/evidence/date-picker-accessible-name-audit-20261020.json`. The filtered RustUI test command compiled the crate but selected zero DatePicker tests; it is not acceptance evidence. The shared target reported E0432 against stale Headstart metadata, while an isolated locked/offline RustUI all-targets check passed after rebuilding `rustui-i18n`. No GUI acceptance was run.
## Implementation — 2026-10-20 (M235)

The audited capability gap is now addressed through the versioned patch
`patches/approved/gpui-component-date-picker-accessibility-v1.patch`. RustUI's
`DatePickerView::accessibility_label` rejects whitespace-only overrides and
forwards nonblank names to the focused Components DatePicker trigger. With no
override, a nonblank visible label is used; a blank visible label receives no
implicit name. The upstream builder stores the optional name and applies it to
the existing focused `gpui_base::DatePicker` ComboBox root, preserving the
existing renderer and state lifecycle.

The patch and unchanged proposal have SHA-256
`ce1ae916fb402f940e2d1dbdd6c8f8ef3c60ee83ced8ad3890a5cce8a343239e`. Source
preimage SHA-256 is
`e7c8c87c76b253703a955c578f28278212d54545d9c9a62a76892a3140095248`; postimage
SHA-256 is
`aedb052400ef31b2754e58820b54453d1f80c39c0f2ed4548d167b1b18bc3b79`. The
patch helper's `--check` confirmed the postimage. The pristine
`.cargo-checksum.json` remains unchanged at SHA-256
`d5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6`. Cargo
uses the local patched package at the same pinned version 0.7.3; no upstream
revision changed.

The candidate and integrated focused RustUI tests each passed 4/4. Candidate
RustUI and Playground locked/offline all-targets checks passed. An integrated
locked/offline workspace all-targets check exited 0 with existing warnings;
its log is `target/headstart/evidence/date-picker-integrated-workspace-check-20261020.log`.
Focused test records are in
`target/headstart/candidates/date-picker-accessible-name-20261020-r2/evidence/`
and
`target/headstart/candidates/date-picker-main-integration-20261020/evidence/`.

No GUI or native accessibility inspection was run. Verify the focused
ComboBox's announced name and selected-date value with VoiceOver, and inspect
the open calendar popup separately. Cross-platform behavior remains unverified.
# M401 — update retained DatePicker owner binding
When a stable DatePicker identity is rendered with a replacement `Binding`, its retained Change subscription now routes through the latest writer instead of the Binding captured at creation. Focused DatePicker tests passed 5/5. Native GUI, trigger naming, VoiceOver, popup, and cross-platform acceptance remain open. Evidence: `target/headstart/evidence/date-picker-current-binding-test-20261021.log`.

## Source audit — 2026-10-21

The current adapter routes retained `Change` events through the latest binding
writer and clones that writer before invoking it, so a callback-triggered
rerender cannot hold the writer-slot borrow. Upstream `clear()` emits
`Change(DateTime::Single(None))`; controlled-value synchronization uses
`set_date(..., emit = false)`, so an external binding update does not call the
user writer. The trigger name is the explicit nonblank override, otherwise the
nonblank visible label; a whitespace-only override is rejected. No additional
decision-free source defect was found in these value, callback, or accessible
name paths.

The focused suite passed 5/5. Evidence:
`target/headstart/evidence/date-picker-contract-audit-20261021.log`. These unit
tests do not exercise the rendered retained entity or native accessibility.
Native GUI selection/clear and external-value synchronization, VoiceOver name
and selected-value announcements, popup accessibility, keyboard/IME behavior,
and cross-platform acceptance remain unverified.

## Capability audit — 2026-10-21

The current RustUI adapter can supply the stable trigger name without any new
vendored edit or dependency revision because the approved, hash-checked M235
patch is already applied to the local pinned GPUI Components 0.7.3 source. The
unmodified public 0.7.3 builder still cannot name its internal focused trigger;
a clean checkout without the approved patch needs that capability and has no
safe wrapper-only alternative. The Input story exercises the visible-label
fallback, and the D1c specimen exposes the explicit accessible-name override.

The focused RustUI DatePicker suite passed 5/5, the approved patch checker
verified postimage `aedb052400ef31b2754e58820b54453d1f80c39c0f2ed4548d167b1b18bc3b79`,
and the locked/offline Playground all-targets check passed. Cargo reported the
existing `block 0.1.6` future-incompatibility warning. These checks confirm the
source integration, not native announcement behavior. Trigger AX role/name and
selected-date value, VoiceOver speech, popup contents, and cross-platform
acceptance remain unverified. Evidence:
`target/headstart/evidence/date-picker-accessibility-capability-audit-20261021.log`.

## Localized trigger name — 2026-10-21

`DatePickerView::localized_accessibility_label(message_id)` resolves the
trigger's accessible name from RustUI's active message catalog. An explicit
`accessibility_label` remains the fallback when localization is applied after
it; otherwise, the DatePicker's visible label is retained as the fallback.
Applying localization repeatedly changes the message key without discarding
that original fallback. A blank localization result uses a nonblank fallback,
an unresolved key uses the fallback, and a blank visible label does not produce
an implicit accessible name.

Focused DatePicker source tests cover translated names, visible-label and
explicit-label fallbacks, repeated localization, blank fallback behavior, and
blank-key rejection. Native AX name/value inspection and VoiceOver announcement
acceptance remain open as described above. The focused suite passed 12/12;
evidence: `target/headstart/evidence/date-picker-localized-accessible-name-test-20261021.log`.
