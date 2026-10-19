# R1a: Stepper owner-current updates

## Scope

Update the Stepper's decrement and increment actions through
`RenderContext::binding_updater`, so each action reads the bound field from the
owning component when the action runs. Keep the public API and existing
positive-step, ascending-range, saturating-arithmetic, and disabled-button
behavior.

## Behavior and bounds

At callback time, both actions first clamp the owner's value to the configured
range, then apply the positive step with saturating arithmetic and clamp the
result to the range. This preserves the existing directional behavior for
out-of-range values: decrementing a value below the range yields the minimum;
incrementing one above the range yields the maximum. A value beyond the
opposite bound is first brought to that bound before the requested step is
applied. If the result equals the owner's current value, the callback returns
`false`, so no write or component notification is requested.

Button disabled state is still derived from the rendered, clamped value. A
click callback uses current owner state even if the component changed after
rendering; the button's disabled state itself updates on the next render.

## Verification

- Inspected `Binding::update_on` and `RenderContext::binding_updater`.
- Formatted the owned Stepper source with `rustfmt`.
- The parent compile reported `E0308` because `clamp` resolved on `&mut i32`.
  Both callbacks now dereference the owner value before clamping.
- Tests and Cargo commands were not run during this task; the parent is
  rerunning workspace compilation after this correction.
