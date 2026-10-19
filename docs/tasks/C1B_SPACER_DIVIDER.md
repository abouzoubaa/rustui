# C1B: Spacer and Divider

## API

- `Spacer()` creates a flexible view that grows along the parent flex container's
  main axis. `min_length(SpacerAxis, f32)` adds a minimum on the explicitly named
  axis; use the axis that matches the parent's main axis. The length must be
  finite and nonnegative; invalid values panic with a diagnostic.
- `Divider()` creates a horizontal separator. `.vertical()` creates a vertical
  separator.

`Divider` delegates rendering and border color to the pinned GPUI Components
`Separator`, which reads the active GPUI semantic border color. These primitives
do not add a second rendering path or claim GUI acceptance. Dividers are
decorative and do not invent accessible labels.

## Verification boundary

Focused unit tests cover spacer bounds validation and default sizing
contract, plus divider orientation selection. Workspace compilation and visual
layout checks remain with the root task.
