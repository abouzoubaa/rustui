# O3: Controlled Popover

`Popover(id, trigger_label, is_open, content)` presents arbitrary RustUI `View`
content from a themed button. `id` is a required, nonempty stable identity within
its composition scope; `is_open` is a `Binding<bool>`. The optional
`.localized_label(message_key)` resolves the trigger label from the active
RustUI catalog. The first slice uses the pinned GPUI Components Popover's
default anchor, offset, appearance, outside-click dismissal, Escape handling,
and focus lifecycle.

The trigger is the existing RustUI-themed GPUI Components Button. The pinned
Popover requires a `Selectable` native trigger, so arbitrary RustUI trigger
views, placement controls, and custom styling are outside this API slice.
The trigger host uses a flex child so its width follows the button rather than
the containing column.

The pinned Popover binds Space to `Confirm` in its key context. RustUI installs
a `NoAction` Space binding for the `Popover > Input` context so a focused text
field receives spaces without closing the popup. Other focused controls retain
the Popover's Space and Enter behavior; Escape is unaffected.

## Owner and lifecycle contract

When the owner binding is open, RustUI lowers `content` in the **same**
`RenderContext` as the trigger, under a dedicated popover retention scope.
That scope has a separate identity namespace from `Keyed` content. The
resulting GPUI element is passed as a child to the pinned Popover. No deferred
callback captures `RenderContext` or creates a separate RustUI owner. Actions,
bindings, environment readers, localization, and theme resolution in content
therefore use their original owner context. A nested `Component` still has its
own normal component owner.

When closed, content is not lowered. The owner host's normal end-of-frame
retention sweep releases content descendants, including retained child
components. The native Popover manages its own focus restoration and deferred
popup registration. Its open-change callback updates the owner binding only
when the value differs. Pinned `sync_open` applies owner changes without
announcing them back to the callback.

## Acceptance required

- Open through the trigger and through an external bound-state change; close
  through the trigger, outside click, Escape, and external state change.
- Confirm open content draws, receives clicks and keyboard focus, and a button
  action updates the original owner. Confirm a control binding inside content
  updates that owner once, without feedback.
- Change typed environment, locale, and theme while open and confirm the
  content updates. Confirm sibling popovers keep independent identity.
- Close and reopen content containing a stateful child component; check its
  expected mount/unmount and state reset. Remove the owner while open and
  verify native dismissal/focus cleanup.
- Check the native accessibility dialog, trigger name, focus movement, and
  Escape behavior. Source-level role/focus behavior is not screenreader
  acceptance.

The focused unit test covers the open-binding equality guard. Build, native
interaction, visual, and accessibility outcomes must be recorded separately;
this implementation alone does not establish them.

## M36 bounded native review

The final Review.app binary (SHA-256
`69943ebe9d681fe7f50a1b3c39e68195f78cfb4992d48ffaade79d870790c771`)
kept the popup open while a focused TextField received literal spaces in
`Popover multiword note works`. The owner incremented once and a child count
changed from 0 to 1. Closing released the child (mounts 1 / releases 1);
reopening mounted a fresh child (2 / 1) while preserving the owner note and
owner count. The inherited version changed to 1 and French `Langues` appeared
while open. Escape and a Light-appearance outside click dismissed the popup.
Removing the owner from inside the popup removed both popup and trigger
(mounts 3 / releases 3); restoring and reopening created a fresh owner with
an empty note and child count 0 (mounts 4 / releases 3). The trigger appeared
at intrinsic width. Root-observed screenshots supported the owner, child, and
environment values; no screenshot artifact is attached here.

Evidence: `target/headstart/milestone36-popover-gui.json`. Trigger Space/Enter,
complete focus traversal, screenreader behavior, sibling identities, and theme
changes while open remain unaccepted.
