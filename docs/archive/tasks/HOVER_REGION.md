# M59: Native hover region

## Contract

`HoverRegion(id, child).on_enter(action).on_exit(action)` observes GPUI's native
hover transitions for an arbitrary RustUI child subtree. Either callback may be
omitted. The nonempty ID is unique among sibling regions and scopes retained
children, so native input state stays with the correct region. The region adds
no focus target, pointer scheduler, position stream, gesture recognizer, or
accessibility role.

Callbacks accept a component `Action` or a context-safe `AppAction` and route
through the same owner semantics as other RustUI controls. Click-only
`AppAction` values are rejected at view construction. `try_on_enter` and
`try_on_exit` return `HoverActionError` instead of panicking.

GPUI emits true on enter and false on exit, including transitions caused by
layout changes under a stationary pointer. Its default mode suppresses hover
after keyboard input until the next pointer move, mouse down, or touch. No exit
callback is promised when a region is removed; owners that keep a hover flag
must clear it while removing the region. Hover-only essential content needs a
keyboard-accessible path.

## Validation

Module checks cover independent callback storage, click-only rejection, and
identity validation. The Playground story shows enter/exit counts, retained
TextField and child button interaction, conditional removal, and explicit owner
state clearing. Bounded native observations are recorded in
`target/headstart/milestone59-hover-gui.json` using the M58/M59 Review.app bundle
with SHA-256
`639a9799d484394a7deee59a98e2cb896a4c151bd0717fe83e846997ff0084e5`. The run
observed enter/exit through clicks, text entry switching keyboard modality and
exiting hover, a child button action, text retained through removal/restoration,
and explicit owner hover-state clearing when removing while hovered.

Acceptance is bounded: CUA clicks moved the pointer, so pointer-only hover dwell
was not tested. Removal does not promise an exit callback. AppAction callbacks,
nested regions, stale-owner callbacks, and full keyboard, screen-reader, and
platform coverage were not independently GUI-tested. See the precise observations
and limits in the evidence JSON.
The repaired shared M58/M59 suite passed 177 tests with 0 failures across 12
binaries in `target/headstart/milestone58-59-tests-repaired.log`; the earlier
failed compile is preserved in `target/headstart/milestone58-59-tests.log`.
