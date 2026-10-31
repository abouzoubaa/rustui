# O2: Tooltip adapter

`Tooltip(text, content).id(...)` wraps a RustUI child with the pinned GPUI
tooltip interaction and GPUI Components text popover. A stable wrapper ID is
required. `.localized(message_id)` resolves text from the active RustUI message
catalog during lowering. The child is lowered once; the wrapper adds no
explicit width or height. The wrapper exposes the tooltip text as its
accessible description while preserving the child's accessible name. The
pinned GPUI Components popup also uses the tooltip role. Source-level mapping is
covered by a focused regression; native assistive-technology announcement is
still unverified.

## M31 evidence

The Playground menu route reaches the combined Menu and Tooltip gallery. Native
layout and the child Button action were observed (count 0 to 1). The tooltip
popup and dismissal were not verified because available pointer automation did
not provide pointer-only movement with a hover dwell. These attempts do not
establish an adapter failure.

The workspace build and tests passed: 90 tests, 0 failures. Log:
`target/headstart/milestone31-32-tests-fixed.log`.

Native hover display, dismissal, touch long-press, and assistive-technology
announcement remain open. The adapter does not replace the child's accessible
name; its wrapper group now carries the tooltip text as an accessible
description.

## M555 stable identity validation

TooltipView::id rejects blank identities. Focused locked/offline test passed 1/1 in `target/headstart/evidence/m555-tooltip-identity-tests-20261021.log`. Native hover, touch, and assistive-technology acceptance remain open.

## M623 — Reject blank localization keys

`Tooltip::localized` rejects a whitespace-only message key before storing it. The focused Tooltip tests passed 2/2.
