# N4: Toolbar

## Contract

`Toolbar(id, items)` presents an ordered list of RustUI `ToolbarItem::action`
controls through the pinned GPUI Components toolbar. The caller supplies a stable
toolbar ID and unique, nonempty item IDs. Item callbacks use RustUI `ControlAction`
and bind to the current component owner during lowering. A localized item resolves
its message ID from the current RustUI locale on each lowering.

The upstream toolbar owns toolbar semantics and roving arrow-key focus. RustUI
uses its existing themed button adapter for each action. Disabling the toolbar
disables its actions as well as upstream roving focus; disabling one item suppresses
its callback. Item order follows the supplied order, including when it changes.

## Boundaries

- This first slice supports action buttons only. Groups, hosted inputs, sizing,
  arbitrary content and icon-only controls need their own contract.
- Toolbar styling follows the existing RustUI Button resolver and upstream toolbar
  layout. No new theme tokens, scheduler, or focus runtime are added.
- Source and unit tests alone do not establish native acceptance.

## Acceptance

- Unit tests reject empty/duplicate IDs and retain supplied order and disabled state.
- Playground toolbar actions increment and reset a visible count. The unavailable
  action keeps its separate hit counter at zero. Reversing order preserves each
  action's identity and behavior.
- The bounded M40 native review exposed named Increment, Reset, Unavailable,
  and Close actions in the accessibility tree. Increment changed the count from
  0 to 1, Reverse reordered the labels to Close/Unavailable/Reset/Increment,
  and Increment still changed the count from 1 to 2. Clicking Unavailable left
  disabled hits at 0. After disabling the whole toolbar, clicking Increment
  left the count at 2 and disabled hits at 0; re-enabling and Reset returned the
  count to 0. The root observed screenshots for the count, order, disabled,
  and reset steps. Evidence: `target/headstart/milestone40-toolbar-gui.json`,
  binary SHA-256 `e248b54ca0d75ca8b4514b3f6dedd0a024b13cdd96464bd8db72cbcac0c4d78f`.
- Tab and arrow-key focus/navigation, keyboard activation, localized Close in
  another active locale, full screenreader behavior, and broader Light/Dark
  visual acceptance remain pending.
