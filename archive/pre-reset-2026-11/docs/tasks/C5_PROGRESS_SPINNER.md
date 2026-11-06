# C5: Progress and Spinner

`Progress(id, value)` renders the pinned GPUI Components linear progress bar.
The stable `id` keeps its native value transition consistent across renders.
`value` is a finite percentage clamped to 0–100. Its upstream base exposes a
progress-indicator role and numeric range and value to accessibility APIs. Use
`.accessibility_label(operation)` to distinguish multiple progress bars.

`.indeterminate(true)` switches `Progress` to the pinned GPUI Components loading
mode; it defaults to `false`.

```rust
let determinate = Progress("upload", 25.0)
    .accessibility_label("Upload progress");
let loading = Progress("sync", 0.0)
    .indeterminate(true)
    .accessibility_label("Syncing");
```

`Spinner()` renders the pinned GPUI Components animation in a named,
indeterminate progress-indicator wrapper. Its default name is “Loading”; use
`.accessibility_label(operation)` for context. Spinner uses a stable default
element ID; give repeated sibling spinners distinct IDs with `.id(identity)`.

Focused tests cover percentage bounds, finite input, and Spinner name and ID
construction. The M73 Progress mode suite separately passed **3 tests**. The
final M73 workspace run passed **237 tests across 12 binaries** (Cargo exit 0),
and the locked/offline all-targets check exited 0. Its Review build exited 0. In native CUA review, two screenshots showed the
indeterminate fill in visibly different positions; determinate progress changed
from 25 to 50 after “Advance progress”, and both mode labels were present. The
record is `target/headstart/milestone73-progress-gui.json`, the test log is
`target/headstart/milestone73-progress-tests.log`, and the build log is
`target/headstart/milestone73-build.log`.

Reduced-motion behavior is verified in upstream source only; it was not toggled
through the OS or UI. This review does not establish exact pixels, cross-platform
behavior, animation timing/performance, keyboard-only interaction, or full
screen-reader acceptance.
