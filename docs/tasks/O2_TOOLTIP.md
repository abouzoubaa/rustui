# O2: Tooltip adapter

`Tooltip(text, content).id(...)` wraps a RustUI child with the pinned GPUI
tooltip interaction and GPUI Components text popover. A stable wrapper ID is
required. `.localized(message_id)` resolves text from the active RustUI message
catalog during lowering. The child is lowered once; the wrapper adds no
explicit width or height and does not set accessible-name/description
properties. Tooltip text is visual descriptive help only.

## M31 evidence

The Playground menu route reaches the combined Menu and Tooltip gallery. Native
layout and the child Button action were observed (count 0 to 1). The tooltip
popup and dismissal were not verified because available pointer automation did
not provide pointer-only movement with a hover dwell. These attempts do not
establish an adapter failure.

The workspace build and tests passed: 90 tests, 0 failures. Log:
`target/headstart/milestone31-32-tests-fixed.log`.

Native hover display, dismissal, touch long-press, and assistive-technology
behavior remain open. This adapter does not set an accessible name or
description.
