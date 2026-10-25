# Controlled Drawer contract

`Drawer(id, visible: Binding<bool>)` presents a modal physical-edge panel in the
existing GPUI Root. It uses pinned `gpui_base::Sheet` for focus trapping, Escape
and backdrop dismissal. A RustUI RootPlugin owns the presentation per window;
there is no additional renderer, scheduler, reactive runtime or dependency.

## Public contract

- Identity must be nonblank and unique within the caller's retention scope.
- `.edge(DrawerEdge::{Left,Right,Top,Bottom})` uses physical edges, including RTL.
  Right and 350 logical pixels are defaults. `.size` requires finite positive
  values and clips the relevant dimension to the available viewport.
- `visible` is controlled: true opens, false closes. True-to-true owner updates
  refresh title, name, content, bindings, edge and size without replacing the
  presentation, focus handle or retained children.
- `.title`, `.accessibility_label`, `.close_label` and their localized builders
  reject blank literal values/keys. Missing/blank/untranslated localized values
  fall back to nonblank literals. The default Close localization key is
  `rustui-action-close` with a literal `Close` fallback. The actual panel has a
  named Dialog role and a named visible Close button.
- Escape, backdrop and Close set owner visibility false exactly once through a
  shared identity/generation-guarded close path. Owner-driven close uses the same
  teardown without writing the already-false binding.
- Prior focus is captured only when opening and restored on close if focus still
  belongs to that presentation and its target remains alive. Unmount removes only
  the owned presentation, preserving the owner's requested state; remount uses
  current owner state. Window close safely releases the host and content.

## Drawer mutual exclusion and compatibility

A new Drawer is rejected while any native Sheet, native Dialog or another Drawer
is active. Its still-current visibility request is reset to false, and the optional
`.presentation_error(Binding<Option<DrawerError>>)` records `ModalConflict` or
`MissingRootHost`. A successful new open clears an old error. Rejection never
closes or replaces another presentation.

Existing Sheet/Dialog trigger paths add only a Drawer-active guard. Dialog keeps
its original active-Dialog guard and can still open above a native Sheet. Sheet
retains its original native replacement/coexistence behavior, including replacing
a Sheet while a native Dialog is active. No new first-active policy is imposed
between already implemented native components.

Drawer tokens include the owner lease entity, public identity and a monotonic
generation. Delayed rejected-open updates also check the caller render revision.
Stale Drawer-owned release and close callbacks cannot close a successor. The host
holds only a weak caller lease. Existing native Sheet/Dialog presentations retain
their pinned upstream event ownership; RustUI never uses an unconditional
`close_sheet` or `close_dialog` as identity evidence.

Direct upstream modal calls bypassing RustUI's Drawer-active guard remain outside
its coordinated open API. If one appears while Drawer is active, Drawer removes
its own layer without closing the foreign modal or restoring focus over it.

## Text-selection scope limitation

The pinned Components WindowState prepares its native/default active text-selection
scope on every Root render. A later Drawer plugin activating an independent scope
would switch scopes twice per frame and clear selection. This candidate therefore
uses the existing Root scope and clears selection once on open and close; it does
not claim a separately isolated Drawer text-selection scope. Focus trapping and
pointer occlusion still come from the existing modal Sheet host. Independent
scope ownership would need an upstream coordination hook rather than a competing
Root-plugin override. Copy/selection and popup stacking remain acceptance gates.

## Caller ownership and lifecycle

Content is lowered with a current `RenderSeed` from its real caller, plus its own
retained-entity store. It is not lowered under an artificial SheetContent owner.
Parent bindings/actions, environment and locale are refreshed on every owner
render. Removing the lease schedules guarded cleanup through its window handle;
expired windows/hosts are harmless. All retained child components and resources
are released with that presentation's store.

## Playground and current checks

The standalone Drawer story and the existing Sheet catalog gallery expose the
public API, editable caller-bound input, live title/size updates, physical edges,
owner close, unmount/remount and competing Sheet/Dialog triggers. The compiled
preview registry adds `drawer-specimen` with the same bounded trigger/title/body
string limits as Sheet, no serialized actions/events and a real local controlled
owner. Imports reject blank labels before factory construction.

The real-Root test harness exercises Escape/focus restoration, original-owner input
binding refresh, stable presentation identity, native modal rejection, preserved Sheet/Dialog combination and replacement, unmount,
stale generations, backdrop/Close and finite-size checks. The preview regression
checks descriptor round trips and blank-import rejection.

The integrating writer ran all seven Drawer tests successfully with the approved
Headstart launcher and persistent production-consolidation target. Playground
library tests and all-target checking also passed. This is source and test-runtime
evidence. Native and browser visual/input/AX, VoiceOver, exact focus behavior,
locale/RTL, window inset handling, popup stacking and cross-platform acceptance
remain pending, as does the separate text-selection scope limitation above.
Focused verification commands:

```sh
./scripts/cargo-headstart test --profile headstart --locked --offline -p rustui --lib drawer
./scripts/cargo-headstart test --profile headstart --locked --offline -p rustui-playground --lib drawer
./scripts/cargo-headstart check --profile headstart --locked --offline -p rustui-playground --all-targets
```
