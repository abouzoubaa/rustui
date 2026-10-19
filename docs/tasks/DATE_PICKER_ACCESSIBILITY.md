# DatePicker accessible name

## Current limitation

The RustUI DatePicker forwards its visible empty-state label through the pinned
GPUI Components `.placeholder(...)` builder. That does not name the focused
control: the component's renderer creates the ComboBox trigger internally with
`gpui_base::DatePicker::new(...)`, sets its role and selected-date value, and
does not apply an accessible label.

The pinned public GPUI Components `DatePicker` builder has no `aria_label`
method or accessible-label field. Its public configuration includes placeholder,
clearability, presets, month count, and appearance. RustUI therefore cannot set
the name on the focused trigger through the supported API. Adding a named
wrapper around the component would name a different node and would not fix the
focused ComboBox.

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
