# DatePicker accessible name

## Current limitation

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
`Role::ComboBox`, expanded state, and the focus handle, but the base `DatePicker`
has no label builder. The base element is private inside the Components element,
so RustUI cannot safely reach it. A wrapper label would name a separate node and
would not fix the focused ComboBox.

## Audit outcome

There is no source-only RustUI fix within the pinned APIs. The proposed patch at
`patches/proposed/gpui-component-date-picker-accessibility.patch` sketches the
needed upstream builder and forwarding change, but it is not an approved,
applied patch and vendored source must remain untouched. Reopen implementation
when that capability is approved and made reproducibly available to this
workspace; then expose the label from the RustUI adapter and add focused source
and native accessibility acceptance.

## Required upstream capability

The pinned component needs an accessible-label builder option stored on its
DatePicker element and forwarded to the internal focused
`gpui_base::DatePicker` ComboBox root. Implement this through an approved,
reproducible versioned upstream patch or an approved dependency revision; do
not modify vendored source directly. RustUI can then pass its existing visible
label, with a separate override only if the product API later requires one.

## Acceptance evidence still needed

- Compile the RustUI adapter against the approved upstream capability.
- Inspect the native accessibility tree and verify the focused trigger exposes
  the ComboBox role and the expected label, both with an empty date and a
  selected date.
- Verify the selected date remains the accessible value and confirm the name
  is announced by VoiceOver. A source-level builder check alone is not native
  accessibility acceptance.
- Inspect the open calendar popup separately; naming the trigger does not
  establish accessible names or announcements for popup contents.
