# Native anchored popup windows proposal

## Current behavior

RustUI's public `Popover` wraps GPUI Components' in-window popover. It renders
inside the parent window and does not create a native child window. GPUI's
pinned common API defines `WindowKind::AnchoredPopup` for a native window
positioned from a parent window, anchor rectangle, gravity, and constraint
adjustment. The pinned macOS backend currently rejects that kind before
opening a window. Its `MacWindow` code selects an `NSPanel` style and popup
level, but does not consume the parent or anchor options.

Relevant pinned source:

- `vendor/gpui-pre-0.3.9/src/platform.rs` — `WindowKind::AnchoredPopup` and
  `PopupOptions` contract.
- `vendor/gpui-pre-macos-0.3.9/src/platform.rs` — macOS rejection path.
- `vendor/gpui-pre-macos-0.3.9/src/window.rs` — panel style/level branches;
  no anchor or parent positioning.
- `crates/rustui/src/components/overlay/popover.rs` — current in-window
  `Popover` adapter.

## Proposed implementation

1. Add a reproducible, versioned patch against the approved pinned
   `gpui-pre 0.3.9` macOS source. Keep the dependency revision unchanged and do
   not edit `vendor/` directly.
2. Implement parent attachment and anchor placement for the existing
   `PopupOptions` contract, including gravity, offset, screen-edge constraints,
   parent movement/close, popup dismissal, and safe teardown.
3. Add an additive RustUI public API for owner-controlled native popup-window
   content. Keep the existing in-window `Popover` behavior as the fallback on
   platforms where native anchored windows are unsupported.
4. Validate the patch against exact preimage hashes, run focused macOS backend
   and RustUI source checks, then verify current-candidate placement, opening,
   dismissal, parent teardown, and owner-state updates in the reusable native
   validation app.

## Approval boundary

No patch is applied to the pinned vendor source, no patch is promoted to
`patches/approved/`, and no dependency revision is changed by this proposal.
`AGENTS.md` requires explicit approval before promoting an upstream patch.
After approval, keep the change as a versioned patch with deterministic apply
and postimage checks, and retain the existing pinned revision.

## Acceptance status

Open. The current-candidate R04 probe verifies error propagation for the
unsupported macOS path. It does not implement or validate native anchored
popup windows. The MVP checklist tracks three separate evidence milestones for
the backend patch, RustUI API/fallback, and native runtime behavior.
