# O6: ContextMenu

`ContextMenu(id, content, items)` wraps any RustUI `View` in an interactive GPUI
element with a required stable, nonempty ID. The pinned GPUI Components
`ContextMenuExt` opens the popup on right-click or long press. It supplies popup
placement, focus, dismissal, and selection behavior. RustUI adds no popup state
or event scheduler.

Items reuse `MenuItem::action`, `MenuItem::separator`, `.localized`, and
`.disabled`. Action IDs must be nonempty and unique within this menu. The same
validation, label resolution, owner-bound callbacks, and explicit disabled
callback guard serve both `Menu` and `ContextMenu`. Labels and disabled flags
are captured when the RustUI owner lowers; an already-open popup is not promised
to refresh live. Native popup row identity remains positional.

The wrapped content lowers under a distinct `ContextMenu` retention scope and
keeps its existing RustUI owner. Removing the wrapper prunes its retained child
slots; action handlers use the host's weak owner update path. Native review
observed dismissal after in-menu owner removal; exact child-release timing
remains unverified.

## Acceptance

- Source tests cover wrapper identity, duplicate action rejection, separators,
  disabled items, and distinct native scope namespaces. They do not replace GUI
  interaction checks.
- The Playground gallery exposes primary left-click, context-menu action counts,
  disabled-hit count, item reorder, locale-dependent Close, and conditional
  removal of the entire child owner. Its `Remove owner` menu item calls the
  gallery owner from inside the open popup; the separate external button also
  removes or restores the target. Restoring that owner starts fresh counters.
- Native review should exercise right-click, long press when available, pointer
  and keyboard selection, disabled suppression, owner removal while the popup is
  open, stale-action no-op, child release, focus and accessibility. Record each
  observed result separately; none is accepted by source construction alone.

## Bounded native review

`target/headstart/milestone43-context-menu-gui.json` records two native Review.app
builds and their verified binary hashes. Right-click opened the popup. Increment
changed the target count to 1; the disabled row received pointer selection but
Return dismissed the popup without incrementing disabled hits. The final build's
in-menu `Remove owner` action removed the target controls and popup, and the
external control restored a fresh owner. Reverse order showed Close, Unavailable,
Reset, Remove owner, Increment; a later Increment yielded count 1 and reversed
true. Removing and restoring again reset count, disabled hits, and reversed to
their defaults. The workspace log `target/headstart/milestone43-45-tests.log`
records 120 passing tests across 12 binaries, with 0 failures.

Detached stale-action invocation, long press, complete keyboard focus and
activation, and full screen-reader behavior remain unverified. Popup dismissal
was observed for the in-menu removal path; exact child-release timing was not.
