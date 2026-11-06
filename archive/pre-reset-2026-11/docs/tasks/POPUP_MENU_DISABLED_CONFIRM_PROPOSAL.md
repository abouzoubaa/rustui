# PopupMenu disabled confirmation proposal

## Scope and source

This is a review-only candidate for the pinned `gpui-component 0.7.3` package from Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab`. It does not change `vendor/`, dependency revisions, or RustUI behavior. The package registry checksum recorded in `Cargo.lock` is `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9`.

The relevant source is `vendor/gpui-component-0.7.3/src/menu/popup_menu.rs`. Its pre-proposal SHA-256 is `3e2e2b4a3d6fe3ced0a764adffe5afc6b0196500286fd67712767ea2cb0954f2`. In `PopupMenu::confirm`, the `ElementItem` arm invokes the handler or action and then calls `dismiss` without checking `disabled`. The standard `Item` arm has the same omission; disabled submenu entries already fall through without confirmation or dismissal.

## Proposed change

`gpui-component-popup-menu-disabled-confirm.patch` adds one guard at the start of `confirm`. When the selected entry is disabled, it returns before invoking a handler/action or dismissing the menu. The guard covers `Item`, `ElementItem`, and `Submenu` so all item kinds honor the same confirmation boundary; enabled-item behavior is unchanged.

Patch SHA-256: `777bc778991c33279cfaa380817671b3b9c1550ecbcb88e8d551acda4304d7e0`.

RustUI can guard a callback it owns, but the public API does not let it suppress `PopupMenu`'s subsequent dismissal. The behavior must be fixed where the component owns both callback dispatch and dismissal.

## Focused validation plan

In an isolated copy of the exact pinned package:

1. Verify the recorded package revision/checksum and pristine source hash; apply this patch with zero fuzz and confirm the hunk matches.
2. Add/run focused component tests showing disabled `ElementItem` handler and action are not called and the menu remains open; cover disabled standard items too.
3. Verify enabled `ElementItem` handler and action still run once and dismiss, and enabled standard-item behavior remains unchanged.
4. Compile the pinned component crate and run its focused tests. GUI verification is needed only if the headless menu tests cannot establish the open/dismiss state.

No patch application, build, test, or GUI validation has been run for this proposal.

## Alternatives and trade-offs

- **RustUI callback guard:** smallest local workaround, but cannot prevent the component from dismissing afterward and leaves other consumers exposed.
- **Guard only `ElementItem`:** smallest narrowly targeted upstream edit, but leaves the same bug in standard `Item`; the proposed shared guard adds only a few lines and makes disabled confirmation consistent across item kinds.
- **Change disabled selection/navigation:** larger behavioral change that affects keyboard selection and rendering. This proposal only makes confirmation inert and leaves navigation policy unchanged.

## Applicability, fallback, and removal

This patch applies only to the exact `gpui-component 0.7.3` source preimage and must be rejected if that source hash or context differs. Keep it proposed until a separately approved isolated candidate passes the focused checks. If rejected or incompatible, RustUI may suppress its own callback as a partial workaround, but must treat prevention of dismissal as unsupported. Remove the patch if upstream adopts equivalent disabled confirmation semantics or the pinned component revision changes.
