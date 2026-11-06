# O4: Toast

## Contract

`Toast(message)` builds an in-app notification using the pinned GPUI Components
notification implementation. The descriptor supports a title, semantic kind,
stable replacement key, auto-hide control, body-click action, and a native
action button. Call `.show()` and pass its `AppAction` to a RustUI control.

The action posts through `WindowExt::push_notification` on the window that
dispatches it.
RustUI already wraps each application window in the GPUI Components `Root`,
which owns the notification list, overlay placement, transition, dismissal, and
notification action lifecycle. The component does not add a timer, overlay
stack, or notification store.

## Example

```rust
Button(
    "Show toast",
    Toast("Your changes were saved.")
        .title("Saved")
        .kind(ToastKind::Success)
        .id("save-result")
        .show(),
)
```

## Boundaries

- Delivery is explicitly in-app only; operating-system notification delivery is not exposed.
- Stable replacement keys are scoped to one application window's notification list.
- GPUI Components controls native animation and dismissal semantics. A toast
  with an action button remains visible until dismissed by the user.
- GUI acceptance is tracked separately from source-level tests; see the bounded
  native evidence below.

## Playground acceptance controls

The Feedback gallery has one persistent success toast with a body-click callback
and an Undo action. Separate callback counters verify that the Undo action does
not invoke the body callback; clicking the body increments its counter and
dismisses the toast. The action toast uses `.autohide(false)` and a stable ID, so
repeated activation exercises same-window replacement. Four kind buttons share
one stable ID and cover Info, Success, Warning, and Error replacement.

## Native GUI evidence

Milestone 39 was reviewed in `target/headstart/verification/M39 M42 Review 2.app`.
The local executable SHA-256 is
`e248b54ca0d75ca8b4514b3f6dedd0a024b13cdd96464bd8db72cbcac0c4d78f`.
The persistent toast visibly showed `Saved`, `Saved changes`, and `Undo`; the
native accessibility tree named the action button `Undo`. Activating Undo
produced action count 1 and body count 0 while leaving the toast visible.
Activating the body then produced body count 1 and action count 1 and dismissed
the toast. The structured record is
`target/headstart/milestone39-toast-gui.json`.

An earlier build displayed a blank Undo action and unnamed action buttons. The
action label and accessibility name fix preceded the reviewed Review 2 binary;
the older-build defect is not attributed to that binary. Variant rendering,
same-ID replacement, close-button behavior, keyboard/focus behavior, and full
screen-reader acceptance remain pending.
