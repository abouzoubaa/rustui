# O1: Menu dropdown adapter

`Menu(label, items).id(...)` lowers to the pinned GPUI Components dropdown menu.
The trigger requires an explicit stable ID. Each action item requires a
nonempty, unique semantic ID within its menu; separators have no action ID.
Action labels may be literal or resolved from the active RustUI message catalog,
and disabled state is forwarded to the native popup item.

Actions are bound to the RustUI owner during lowering. The popup captures the
lowered labels, disabled flags, and owner-bound handlers for that popup
instance; changes appear when the popup is rebuilt after dismissal. Upstream
owns popup lifecycle, focus, and keyboard behavior. Its pinned API does not
accept RustUI IDs for native popup rows, so item IDs remain semantic RustUI
identities and native row identity remains positional.

## Executed acceptance (M30)

The native M30 Final Review run opened the popup and displayed its separator.
Keyboard Increment changed Actions to 1; pointer Increment changed it to 2;
pointer Reset restored 0. Disabled pointer selection followed by Enter left
disabled action hits at 0. The ordinary Button Increment smoke check changed
Count from 0 to 1 after extraction of the shared theme helper. The final popup
aligned below its intrinsic-width trigger.

The workspace unit/integration command passed 86 tests across 12 binaries,
with zero failures. Exact log: `target/headstart/milestone30-menu-tests-accepted.log`.
Native outcomes and binary hash: `target/headstart/milestone30-menu-gui.json`.
The evidence record identifies the continuation handoff as its native provenance.

## Remaining limits

The pinned popup still permits selecting a disabled row by keyboard, and Enter
dismisses it; RustUI suppresses that row's callback. Native row identity remains
positional. Open-popup owner removal, live label refresh and complete accessibility
acceptance remain unverified. Submenus are not implemented by this adapter.


## M100 — native owner-removal acceptance (2026-10-19)

The canonical `dev.rustui.playground` app opened the Actions popup. Clicking the story's **Remove/restore menu owner** control removed the Menu view and dismissed the popup immediately. A fresh accessibility snapshot showed `Menu owner removed` and no menu/dialog node. A stale callback could not be activated in the observed state; destruction/release of the story component entity was not measured.

Evidence: `target/headstart/evidence/menu-owner-removal-20261019.json`; build log: `target/headstart/menu-owner-playground-build.log`. The app was quit, the disposable bundle and executable were removed, no Playground process remained, and shared Cargo caches were retained.
